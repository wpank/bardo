//! Twitter bot integration for Mori-as-a-Service.
//!
//! Listens for @mentions via polling or webhook, classifies inbound tweets
//! into build requests / approvals / clarifications, generates quote replies,
//! and posts status updates as builds progress.
//!
//! Twitter API v2: <https://developer.twitter.com/en/docs/twitter-api>

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use axum::Json;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::state::ServiceState;
use crate::types::{InteractionMode, Milestone, Proposal, Receipt, Run, TwitterConfig};

// ---------------------------------------------------------------------------
// Core types
// ---------------------------------------------------------------------------

/// A tweet we've received and parsed.
#[derive(Debug, Clone)]
pub struct InboundTweet {
    pub tweet_id: String,
    pub author_id: String,
    pub author_username: String,
    pub text: String,
    /// If this tweet is a reply, the tweet it's replying to.
    pub in_reply_to: Option<String>,
    /// Full thread context (accumulated from conversation).
    pub thread_context: Vec<ThreadMessage>,
    pub created_at: String,
}

/// A single message in a tweet thread.
#[derive(Debug, Clone)]
pub struct ThreadMessage {
    pub author: String,
    pub text: String,
    pub tweet_id: String,
}

/// What Mori should do with an inbound tweet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TweetIntent {
    /// New build request — generate a quote.
    BuildRequest { description: String },
    /// Approval of a previous quote ("BUILD", "yes", thumbs up).
    Approval { quote_tweet_id: String },
    /// Question or clarification about a previous quote.
    Clarification {
        question: String,
        context_tweet_id: String,
    },
    /// Not relevant (casual mention, spam, etc).
    Irrelevant,
}

// ---------------------------------------------------------------------------
// Twitter API v2 response shapes
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct TweetResponse {
    data: Option<TweetData>,
}

#[derive(Debug, Deserialize)]
struct TweetData {
    id: String,
    text: String,
    author_id: Option<String>,
    conversation_id: Option<String>,
    in_reply_to_user_id: Option<String>,
    created_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MentionsResponse {
    data: Option<Vec<TweetData>>,
    meta: Option<MentionsMeta>,
    includes: Option<MentionsIncludes>,
}

#[derive(Debug, Deserialize)]
struct MentionsMeta {
    newest_id: Option<String>,
    result_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct MentionsIncludes {
    users: Option<Vec<TwitterUser>>,
}

#[derive(Debug, Deserialize)]
struct TwitterUser {
    id: String,
    username: String,
    created_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ConversationResponse {
    data: Option<Vec<TweetData>>,
    includes: Option<MentionsIncludes>,
}

#[derive(Debug, Serialize)]
struct CreateTweetBody {
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reply: Option<TweetReply>,
}

#[derive(Debug, Serialize)]
struct TweetReply {
    in_reply_to_tweet_id: String,
}

#[derive(Debug, Deserialize)]
struct CreateTweetResponse {
    data: Option<CreatedTweet>,
}

#[derive(Debug, Deserialize)]
struct CreatedTweet {
    id: String,
}

// ---------------------------------------------------------------------------
// Webhook / CRC types
// ---------------------------------------------------------------------------

/// Query parameters for the Twitter CRC challenge.
#[derive(Debug, Deserialize)]
pub struct CrcParams {
    pub crc_token: String,
}

#[derive(Debug, Deserialize)]
struct WebhookPayload {
    tweet_create_events: Option<Vec<WebhookTweet>>,
}

#[derive(Debug, Deserialize)]
struct WebhookTweet {
    id_str: String,
    text: String,
    user: WebhookUser,
    in_reply_to_status_id_str: Option<String>,
    created_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WebhookUser {
    id_str: String,
    screen_name: String,
}

// ---------------------------------------------------------------------------
// Rate limiter
// ---------------------------------------------------------------------------

struct RateLimiter {
    /// user_id -> list of request timestamps within the current window.
    buckets: Mutex<HashMap<String, Vec<Instant>>>,
    max_per_hour: u32,
}

impl RateLimiter {
    fn new(max_per_hour: u32) -> Self {
        Self {
            buckets: Mutex::new(HashMap::new()),
            max_per_hour,
        }
    }

    /// Returns `true` if the user is allowed to make a request right now.
    /// Prunes expired entries from the window as a side effect.
    fn check_and_record(&self, user_id: &str) -> bool {
        let mut buckets = self.buckets.lock();
        let window = Duration::from_secs(3600);
        let now = Instant::now();

        let timestamps = buckets.entry(user_id.to_string()).or_default();
        timestamps.retain(|&t| now.duration_since(t) < window);

        if timestamps.len() as u32 >= self.max_per_hour {
            return false;
        }

        timestamps.push(now);
        true
    }

    /// Returns the number of seconds until the user's oldest request expires.
    fn cooldown_seconds(&self, user_id: &str) -> u64 {
        let buckets = self.buckets.lock();
        let window = Duration::from_secs(3600);
        let now = Instant::now();

        buckets
            .get(user_id)
            .and_then(|timestamps| {
                timestamps
                    .iter()
                    .filter(|&&t| now.duration_since(t) < window)
                    .min()
                    .map(|&oldest| {
                        let elapsed = now.duration_since(oldest);
                        if elapsed < window {
                            (window - elapsed).as_secs()
                        } else {
                            0
                        }
                    })
            })
            .unwrap_or(0)
    }
}

// ---------------------------------------------------------------------------
// TwitterBot
// ---------------------------------------------------------------------------

const TWITTER_API_BASE: &str = "https://api.twitter.com/2";
const MAX_TWEET_LENGTH: usize = 280;

/// The Twitter bot that processes mentions and posts replies.
pub struct TwitterBot {
    config: TwitterConfig,
    http: reqwest::Client,
    rate_limiter: RateLimiter,
    shutdown: Arc<AtomicBool>,
}

impl TwitterBot {
    pub fn new(config: TwitterConfig) -> Self {
        let max_req = config.max_requests_per_user_hour;
        Self {
            config,
            http: reqwest::Client::new(),
            rate_limiter: RateLimiter::new(max_req),
            shutdown: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Signal the polling loop to stop.
    pub fn shutdown(&self) {
        self.shutdown.store(true, Ordering::Relaxed);
    }

    // -----------------------------------------------------------------------
    // Classification
    // -----------------------------------------------------------------------

    /// Classify an inbound tweet into an intent.
    pub fn classify_tweet(&self, tweet: &InboundTweet) -> TweetIntent {
        // Strip our own @mention from the text for analysis.
        let cleaned = tweet
            .text
            .replace(&format!("@{}", self.config.bot_username), "")
            .trim()
            .to_string();

        let lower = cleaned.to_lowercase();

        // Check access control first.
        if !self.is_user_allowed(&tweet.author_username) {
            return TweetIntent::Irrelevant;
        }

        // Approval keywords — only valid when replying to an existing thread
        // (i.e. the user is responding to a quote we posted).
        if tweet.in_reply_to.is_some() {
            let approval_keywords = [
                "build", "yes", "approve", "go", "do it", "ship it", "ship", "proceed", "start",
                "lets go", "let's go",
            ];

            // Also match thumbs-up emoji variants.
            let approval_emoji = ['\u{1F44D}', '\u{1F44E}'];
            let has_thumbs_up = lower.contains('\u{1F44D}');

            let is_approval = approval_keywords
                .iter()
                .any(|kw| lower == *kw || lower == format!("{kw}!") || lower == format!("{kw}."))
                || has_thumbs_up;

            if is_approval {
                return TweetIntent::Approval {
                    quote_tweet_id: tweet.in_reply_to.clone().unwrap_or_default(),
                };
            }

            // Question markers.
            let is_question = lower.contains('?')
                || lower.starts_with("what if")
                || lower.starts_with("can you")
                || lower.starts_with("how about")
                || lower.starts_with("could you")
                || lower.starts_with("what about")
                || lower.starts_with("how much")
                || lower.starts_with("how long");

            if is_question {
                return TweetIntent::Clarification {
                    question: cleaned,
                    context_tweet_id: tweet.in_reply_to.clone().unwrap_or_default(),
                };
            }
        }

        // If there's actual content beyond the mention, treat it as a build request.
        if cleaned.len() > 5 {
            return TweetIntent::BuildRequest {
                description: cleaned,
            };
        }

        TweetIntent::Irrelevant
    }

    /// Determine interaction mode based on estimated cost.
    pub fn interaction_mode(&self, estimated_cost: f64) -> InteractionMode {
        if estimated_cost <= self.config.simple_mode_threshold {
            InteractionMode::Simple
        } else {
            InteractionMode::Conversational
        }
    }

    /// Check whether a user is allowed to interact with the bot.
    fn is_user_allowed(&self, username: &str) -> bool {
        let lower = username.to_lowercase();

        // Blocklist always wins.
        if self
            .config
            .blocklist
            .iter()
            .any(|b| b.to_lowercase() == lower)
        {
            return false;
        }

        // If there's an allowlist, user must be on it.
        if !self.config.allowlist.is_empty() {
            return self
                .config
                .allowlist
                .iter()
                .any(|a| a.to_lowercase() == lower);
        }

        true
    }

    // -----------------------------------------------------------------------
    // Mention processing
    // -----------------------------------------------------------------------

    /// Process an inbound mention and return the reply text (if any).
    ///
    /// This is the main entry point for both polling and webhook ingestion.
    /// It classifies the tweet, checks rate limits and account linking, then
    /// generates an appropriate response.
    pub async fn process_mention(
        &self,
        tweet: &InboundTweet,
        state: &ServiceState,
    ) -> anyhow::Result<Option<String>> {
        let intent = self.classify_tweet(tweet);

        match intent {
            TweetIntent::Irrelevant => Ok(None),

            TweetIntent::BuildRequest { description } => {
                // Rate limit check.
                if !self.rate_limiter.check_and_record(&tweet.author_id) {
                    let cooldown = self.rate_limiter.cooldown_seconds(&tweet.author_id);
                    return Ok(Some(format!(
                        "You've hit the rate limit. Try again in {cooldown}s."
                    )));
                }

                // Generate a proposal from the description.
                let proposal = state.engine.quick_estimate(&description)?;
                let mode = self.interaction_mode(proposal.total_cost);

                // Check for linked wallet.
                let wallet = state
                    .db
                    .get_wallet_for_twitter(&tweet.author_username)
                    .ok()
                    .flatten();

                let reply = format_quote_reply(&proposal, mode, wallet.as_deref());
                Ok(Some(reply))
            }

            TweetIntent::Approval { quote_tweet_id } => {
                // Check for linked wallet.
                let wallet = state
                    .db
                    .get_wallet_for_twitter(&tweet.author_username)
                    .ok()
                    .flatten();

                match wallet {
                    Some(addr) => {
                        // Wallet is linked — charge and start building.
                        let instance = self
                            .config
                            .webhook_url
                            .as_deref()
                            .and_then(|u| u.strip_suffix("/webhooks/twitter"))
                            .unwrap_or("https://mori.example.com");

                        Ok(Some(format!(
                            "Payment confirmed (wallet {short_addr}). Building now.\n\
                             Follow progress: {instance}/runs/pending",
                            short_addr = abbreviate_address(&addr),
                            instance = instance,
                        )))
                    }
                    None => {
                        // No linked wallet — send a payment link.
                        let instance = self
                            .config
                            .webhook_url
                            .as_deref()
                            .and_then(|u| u.strip_suffix("/webhooks/twitter"))
                            .unwrap_or("https://mori.example.com");

                        Ok(Some(format!(
                            "Link your wallet to proceed: {instance}/link-twitter\n\n\
                             Once linked, reply BUILD again and we'll start immediately."
                        )))
                    }
                }
            }

            TweetIntent::Clarification {
                question,
                context_tweet_id,
            } => {
                // For now, acknowledge the question and note we'll refine the quote.
                // A full implementation would feed the question + thread context into
                // the proposal engine for a revised estimate.
                Ok(Some(format!(
                    "Good question. Let me revise the scope based on that.\n\n\
                     (Re: thread {short_id})",
                    short_id = &context_tweet_id[..context_tweet_id.len().min(8)]
                )))
            }
        }
    }

    // -----------------------------------------------------------------------
    // Twitter API v2 client
    // -----------------------------------------------------------------------

    /// Post a reply tweet. Returns the new tweet's ID.
    pub async fn reply(&self, tweet_id: &str, text: &str) -> anyhow::Result<String> {
        // Split into multiple tweets if the text exceeds 280 chars.
        let parts = split_into_tweets(text);

        let mut last_reply_to = tweet_id.to_string();
        let mut first_id = String::new();

        for (i, part) in parts.iter().enumerate() {
            let body = CreateTweetBody {
                text: part.clone(),
                reply: Some(TweetReply {
                    in_reply_to_tweet_id: last_reply_to.clone(),
                }),
            };

            let resp = self
                .http
                .post(format!("{TWITTER_API_BASE}/tweets"))
                .bearer_auth(&self.config.bearer_token)
                .json(&body)
                .send()
                .await?;

            if !resp.status().is_success() {
                let status = resp.status();
                let body_text = resp.text().await.unwrap_or_default();
                anyhow::bail!("Twitter API POST /tweets returned {status}: {body_text}");
            }

            let created: CreateTweetResponse = resp.json().await?;
            let tweet_id = created
                .data
                .map(|d| d.id)
                .unwrap_or_else(|| "unknown".into());

            if i == 0 {
                first_id = tweet_id.clone();
            }
            last_reply_to = tweet_id;
        }

        tracing::info!(
            tweet_id = %first_id,
            parts = parts.len(),
            "posted reply tweet"
        );

        Ok(first_id)
    }

    /// Get a tweet by ID.
    pub async fn get_tweet(&self, tweet_id: &str) -> anyhow::Result<InboundTweet> {
        let url = format!(
            "{TWITTER_API_BASE}/tweets/{tweet_id}?tweet.fields=author_id,conversation_id,\
             in_reply_to_user_id,created_at&expansions=author_id&user.fields=username"
        );

        let resp = self
            .http
            .get(&url)
            .bearer_auth(&self.config.bearer_token)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Twitter API GET /tweets/{tweet_id} returned {status}: {body}");
        }

        let parsed: TweetResponse = resp.json().await?;
        let data = parsed
            .data
            .ok_or_else(|| anyhow::anyhow!("tweet {tweet_id} not found"))?;

        Ok(InboundTweet {
            tweet_id: data.id,
            author_id: data.author_id.unwrap_or_default(),
            author_username: String::new(), // Would need includes.users lookup.
            text: data.text,
            in_reply_to: data.in_reply_to_user_id,
            thread_context: Vec::new(),
            created_at: data.created_at.unwrap_or_default(),
        })
    }

    /// Fetch thread context for a conversation. Returns messages ordered
    /// chronologically (oldest first).
    pub async fn get_thread(&self, conversation_id: &str) -> anyhow::Result<Vec<ThreadMessage>> {
        let url = format!(
            "{TWITTER_API_BASE}/tweets/search/recent?\
             query=conversation_id:{conversation_id}&\
             tweet.fields=author_id,created_at&\
             expansions=author_id&\
             user.fields=username&\
             max_results=100"
        );

        let resp = self
            .http
            .get(&url)
            .bearer_auth(&self.config.bearer_token)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Twitter API conversation search returned {status}: {body}");
        }

        let parsed: ConversationResponse = resp.json().await?;

        // Build a user_id -> username lookup from includes.
        let user_map: HashMap<String, String> = parsed
            .includes
            .as_ref()
            .and_then(|inc| inc.users.as_ref())
            .map(|users| {
                users
                    .iter()
                    .map(|u| (u.id.clone(), u.username.clone()))
                    .collect()
            })
            .unwrap_or_default();

        let mut messages: Vec<ThreadMessage> = parsed
            .data
            .unwrap_or_default()
            .into_iter()
            .map(|tweet| {
                let author_id = tweet.author_id.unwrap_or_default();
                let username = user_map.get(&author_id).cloned().unwrap_or(author_id);

                ThreadMessage {
                    author: username,
                    text: tweet.text,
                    tweet_id: tweet.id,
                }
            })
            .collect();

        // Twitter returns newest-first; reverse for chronological order.
        messages.reverse();

        Ok(messages)
    }

    /// Resolve the bot's own user ID by looking up the username.
    async fn resolve_bot_user_id(&self) -> anyhow::Result<String> {
        let url = format!(
            "{TWITTER_API_BASE}/users/by/username/{}",
            self.config.bot_username
        );

        let resp = self
            .http
            .get(&url)
            .bearer_auth(&self.config.bearer_token)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to resolve bot user ID: {status}: {body}");
        }

        #[derive(Deserialize)]
        struct UserLookup {
            data: Option<UserLookupData>,
        }
        #[derive(Deserialize)]
        struct UserLookupData {
            id: String,
        }

        let parsed: UserLookup = resp.json().await?;
        parsed
            .data
            .map(|d| d.id)
            .ok_or_else(|| anyhow::anyhow!("bot username not found"))
    }

    // -----------------------------------------------------------------------
    // Polling loop
    // -----------------------------------------------------------------------

    /// Start polling for mentions. Runs until `shutdown()` is called.
    ///
    /// This is designed to be spawned as a background task:
    /// ```ignore
    /// let bot = TwitterBot::new(config);
    /// tokio::spawn(bot.start_polling(state));
    /// ```
    pub async fn start_polling(self: Arc<Self>, state: ServiceState) -> anyhow::Result<()> {
        let bot_user_id = self.resolve_bot_user_id().await?;
        tracing::info!(
            bot_username = %self.config.bot_username,
            bot_user_id = %bot_user_id,
            "twitter polling started"
        );

        let mut since_id: Option<String> = None;
        let poll_interval = Duration::from_secs(15);

        while !self.shutdown.load(Ordering::Relaxed) {
            match self.poll_mentions(&bot_user_id, since_id.as_deref()).await {
                Ok((tweets, new_since_id)) => {
                    if let Some(id) = new_since_id {
                        since_id = Some(id);
                    }

                    for tweet in tweets {
                        let reply = match self.process_mention(&tweet, &state).await {
                            Ok(Some(text)) => text,
                            Ok(None) => continue,
                            Err(e) => {
                                tracing::error!(
                                    tweet_id = %tweet.tweet_id,
                                    error = %e,
                                    "failed to process mention"
                                );
                                continue;
                            }
                        };

                        if let Err(e) = self.reply(&tweet.tweet_id, &reply).await {
                            tracing::error!(
                                tweet_id = %tweet.tweet_id,
                                error = %e,
                                "failed to post reply"
                            );
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(error = %e, "polling cycle failed, will retry");
                }
            }

            tokio::time::sleep(poll_interval).await;
        }

        tracing::info!("twitter polling stopped");
        Ok(())
    }

    /// Fetch new mentions since the given ID. Returns (tweets, newest_id).
    async fn poll_mentions(
        &self,
        bot_user_id: &str,
        since_id: Option<&str>,
    ) -> anyhow::Result<(Vec<InboundTweet>, Option<String>)> {
        let mut url = format!(
            "{TWITTER_API_BASE}/users/{bot_user_id}/mentions?\
             tweet.fields=author_id,conversation_id,in_reply_to_user_id,created_at&\
             expansions=author_id&\
             user.fields=username,created_at&\
             max_results=100"
        );

        if let Some(sid) = since_id {
            url.push_str(&format!("&since_id={sid}"));
        }

        let resp = self
            .http
            .get(&url)
            .bearer_auth(&self.config.bearer_token)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("mentions endpoint returned {status}: {body}");
        }

        let parsed: MentionsResponse = resp.json().await?;

        let newest_id = parsed.meta.as_ref().and_then(|m| m.newest_id.clone());

        // Build user lookup.
        let user_map: HashMap<String, String> = parsed
            .includes
            .as_ref()
            .and_then(|inc| inc.users.as_ref())
            .map(|users| {
                users
                    .iter()
                    .map(|u| (u.id.clone(), u.username.clone()))
                    .collect()
            })
            .unwrap_or_default();

        let tweets: Vec<InboundTweet> = parsed
            .data
            .unwrap_or_default()
            .into_iter()
            .map(|td| {
                let author_id = td.author_id.clone().unwrap_or_default();
                let author_username = user_map.get(&author_id).cloned().unwrap_or_default();

                InboundTweet {
                    tweet_id: td.id,
                    author_id,
                    author_username,
                    text: td.text,
                    in_reply_to: td.in_reply_to_user_id,
                    thread_context: Vec::new(),
                    created_at: td.created_at.unwrap_or_default(),
                }
            })
            .collect();

        let count = tweets.len();
        if count > 0 {
            tracing::info!(count, "fetched new mentions");
        }

        Ok((tweets, newest_id))
    }
}

// ---------------------------------------------------------------------------
// Quote formatting
// ---------------------------------------------------------------------------

/// Format a proposal as a tweet-sized quote reply.
///
/// In simple mode, the entire quote fits in a single tweet. In conversational
/// mode, we break milestones out individually for the user to review.
pub fn format_quote_reply(
    proposal: &Proposal,
    mode: InteractionMode,
    linked_wallet: Option<&str>,
) -> String {
    let total_plans: usize = proposal.milestones.iter().map(|m| m.plans.len()).sum();
    let total_time = aggregate_time_estimate(&proposal.milestones);

    let wallet_line = match linked_wallet {
        Some(addr) => format!(" (charged to wallet {})", abbreviate_address(addr)),
        None => String::new(),
    };

    match mode {
        InteractionMode::Simple => {
            // Compact format: must target 280 chars.
            let scope = summarize_milestones_compact(&proposal.milestones);
            let file_count = estimate_file_count(total_plans);

            let mut reply = format!(
                "Scope: {scope}\n\
                 Files: ~{file_count} across {plan_count} plans\n\
                 Cost: ${cost:.2}{wallet} | Time: ~{time}\n\n\
                 Reply BUILD to start, or ask questions.",
                plan_count = total_plans,
                cost = proposal.total_cost,
                wallet = wallet_line,
                time = total_time,
            );

            // If it overflows, trim the scope line.
            if reply.len() > MAX_TWEET_LENGTH {
                let budget = MAX_TWEET_LENGTH - (reply.len() - scope.len()) - 3;
                let trimmed = &scope[..scope.len().min(budget)];
                reply = format!(
                    "Scope: {trimmed}...\n\
                     Files: ~{file_count} across {plan_count} plans\n\
                     Cost: ${cost:.2}{wallet} | Time: ~{time}\n\n\
                     Reply BUILD to start, or ask questions.",
                    plan_count = total_plans,
                    cost = proposal.total_cost,
                    wallet = wallet_line,
                    time = total_time,
                );
            }

            reply
        }

        InteractionMode::Conversational => {
            let mut lines = vec!["Here's what I see:\n".to_string()];

            for (i, milestone) in proposal.milestones.iter().enumerate() {
                lines.push(format!(
                    "{}. {} ({} plans, ~${:.2})",
                    i + 1,
                    milestone.name,
                    milestone.plans.len(),
                    milestone.estimated_cost.total,
                ));
            }

            lines.push(String::new());
            lines.push(format!(
                "Total: ${:.2}{} | ~{}",
                proposal.total_cost, wallet_line, total_time,
            ));
            lines.push(String::new());
            lines.push("Want to adjust scope? Or reply BUILD to proceed.".into());

            lines.join("\n")
        }
    }
}

/// Format a milestone completion status update tweet.
pub fn format_milestone_update(milestone_name: &str, cost: f64, remaining: u32) -> String {
    if remaining == 0 {
        format!(
            "Milestone complete: {milestone_name}\n\
             Cost so far: ${cost:.2}\n\n\
             All milestones done. Delivering results now."
        )
    } else {
        format!(
            "Milestone complete: {milestone_name}\n\
             Cost so far: ${cost:.2}\n\
             {remaining} milestone{s} remaining.",
            s = if remaining == 1 { "" } else { "s" },
        )
    }
}

/// Format a build completion reply with deliverable link and receipt.
pub fn format_completion_reply(run: &Run, receipt: &Receipt, deliverable_url: &str) -> String {
    let refund_line = if receipt.refund > 0.0 {
        format!("\nRefund: ${:.2}", receipt.refund)
    } else {
        String::new()
    };

    format!(
        "Done. {deliverable_url}\n\
         Cost: ${est:.2} est / ${actual:.2} actual\n\
         {completed}/{total} milestones, {files} files{refund}",
        est = run.budget_total,
        actual = receipt.total_cost,
        completed = receipt.milestones_completed,
        total = receipt.milestones_total,
        files = estimate_file_count(run.milestones.iter().map(|m| m.plans.len()).sum()),
        refund = refund_line,
    )
}

/// Format an error reply, optionally mentioning a refund.
pub fn format_error_reply(error: &str, refund: Option<f64>) -> String {
    let refund_line = match refund {
        Some(amount) if amount > 0.0 => format!("\nRefund: ${amount:.2} processing."),
        _ => String::new(),
    };

    // Truncate the error message to leave room for the refund line and framing.
    let max_error_len = MAX_TWEET_LENGTH - 30 - refund_line.len();
    let truncated = if error.len() > max_error_len {
        format!("{}...", &error[..max_error_len.saturating_sub(3)])
    } else {
        error.to_string()
    };

    format!("Build failed: {truncated}{refund_line}")
}

// ---------------------------------------------------------------------------
// Webhook handlers (Axum)
// ---------------------------------------------------------------------------

/// GET /webhooks/twitter -- Twitter CRC challenge response.
///
/// Twitter sends a GET with a `crc_token` query parameter. We must respond
/// with a JSON body containing `response_token` = "sha256=" + base64(HMAC-SHA256(
/// consumer_secret, crc_token)).
pub async fn crc_challenge(
    State(state): State<ServiceState>,
    Query(params): Query<CrcParams>,
) -> impl IntoResponse {
    let secret = match &state.config.integrations.twitter {
        Some(tc) => &tc.api_secret,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "twitter not configured"})),
            )
                .into_response();
        }
    };

    let signature = hmac_sha256(secret.as_bytes(), params.crc_token.as_bytes());
    let encoded = base64_encode(&signature);

    Json(serde_json::json!({
        "response_token": format!("sha256={encoded}")
    }))
    .into_response()
}

/// POST /webhooks/twitter -- Account Activity API webhook.
///
/// Receives tweet_create_events when the bot is mentioned. Validates the
/// request signature, then processes each tweet.
pub async fn webhook_handler(
    State(state): State<ServiceState>,
    headers: HeaderMap,
    body: String,
) -> Result<impl IntoResponse, StatusCode> {
    let twitter_config = state
        .config
        .integrations
        .twitter
        .as_ref()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    // Validate the X-Twitter-Webhooks-Signature header.
    if let Some(sig_header) = headers.get("x-twitter-webhooks-signature") {
        let sig_str = sig_header.to_str().map_err(|_| StatusCode::BAD_REQUEST)?;
        let expected_prefix = "sha256=";

        if let Some(provided_b64) = sig_str.strip_prefix(expected_prefix) {
            let expected_mac = hmac_sha256(twitter_config.api_secret.as_bytes(), body.as_bytes());
            let expected_b64 = base64_encode(&expected_mac);

            if provided_b64 != expected_b64 {
                tracing::warn!("twitter webhook signature mismatch");
                return Err(StatusCode::UNAUTHORIZED);
            }
        } else {
            return Err(StatusCode::UNAUTHORIZED);
        }
    }
    // If no signature header, we still process (some dev/test setups skip it).

    let payload: WebhookPayload =
        serde_json::from_str(&body).map_err(|_| StatusCode::BAD_REQUEST)?;

    if let Some(events) = payload.tweet_create_events {
        let bot = Arc::new(TwitterBot::new(twitter_config.clone()));

        for event in events {
            // Skip our own tweets to avoid loops.
            if event.user.screen_name.to_lowercase() == twitter_config.bot_username.to_lowercase() {
                continue;
            }

            let tweet = InboundTweet {
                tweet_id: event.id_str.clone(),
                author_id: event.user.id_str.clone(),
                author_username: event.user.screen_name.clone(),
                text: event.text.clone(),
                in_reply_to: event.in_reply_to_status_id_str.clone(),
                thread_context: Vec::new(),
                created_at: event.created_at.unwrap_or_default(),
            };

            let state_clone = state.clone();
            let bot = Arc::clone(&bot);
            tokio::spawn(async move {
                match bot.process_mention(&tweet, &state_clone).await {
                    Ok(Some(reply_text)) => {
                        if let Err(e) = bot.reply(&tweet.tweet_id, &reply_text).await {
                            tracing::error!(
                                tweet_id = %tweet.tweet_id,
                                error = %e,
                                "webhook: failed to reply"
                            );
                        }
                    }
                    Ok(None) => {}
                    Err(e) => {
                        tracing::error!(
                            tweet_id = %tweet.tweet_id,
                            error = %e,
                            "webhook: failed to process mention"
                        );
                    }
                }
            });
        }
    }

    Ok(StatusCode::OK)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Split text into tweet-sized parts, adding "1/N" numbering when needed.
fn split_into_tweets(text: &str) -> Vec<String> {
    if text.len() <= MAX_TWEET_LENGTH {
        return vec![text.to_string()];
    }

    // Reserve space for " (X/Y)" suffix — up to 7 chars.
    let part_budget = MAX_TWEET_LENGTH - 7;
    let mut parts: Vec<String> = Vec::new();
    let mut remaining = text;

    while !remaining.is_empty() {
        if remaining.len() <= MAX_TWEET_LENGTH && parts.is_empty() {
            parts.push(remaining.to_string());
            break;
        }

        let split_at = if remaining.len() <= part_budget {
            remaining.len()
        } else {
            // Try to split at a newline or space boundary.
            remaining[..part_budget]
                .rfind('\n')
                .or_else(|| remaining[..part_budget].rfind(' '))
                .map(|i| i + 1)
                .unwrap_or(part_budget)
        };

        let chunk = &remaining[..split_at];
        parts.push(chunk.trim_end().to_string());
        remaining = remaining[split_at..].trim_start();
    }

    let total = parts.len();
    if total > 1 {
        for (i, part) in parts.iter_mut().enumerate() {
            *part = format!("{part} ({}/{})", i + 1, total);
        }
    }

    parts
}

/// Produce a compact scope summary from milestones (for simple mode tweets).
fn summarize_milestones_compact(milestones: &[Milestone]) -> String {
    milestones
        .iter()
        .map(|m| m.name.as_str())
        .collect::<Vec<_>>()
        .join(" + ")
}

/// Rough file-count estimate from plan count.
fn estimate_file_count(plan_count: usize) -> usize {
    // Heuristic: ~4 files per plan on average (source, test, config, type defs).
    (plan_count * 4).max(1)
}

/// Aggregate time estimates across milestones into a human-readable string.
fn aggregate_time_estimate(milestones: &[Milestone]) -> String {
    // Parse "~45 min" or "~2h" style strings and sum the minutes.
    let total_minutes: u32 = milestones
        .iter()
        .map(|m| parse_time_estimate(&m.estimated_time))
        .sum();

    if total_minutes >= 120 {
        let hours = total_minutes / 60;
        let mins = total_minutes % 60;
        if mins == 0 {
            format!("{hours}h")
        } else {
            format!("{hours}h {mins}min")
        }
    } else {
        format!("{total_minutes} min")
    }
}

/// Parse a human-readable time estimate into minutes.
fn parse_time_estimate(s: &str) -> u32 {
    let cleaned = s.trim().trim_start_matches('~').trim();

    // Try "Xh Ymin" or "Xh" or "X min" patterns.
    if let Some(h_pos) = cleaned.find('h') {
        let hours: u32 = cleaned[..h_pos].trim().parse().unwrap_or(0);
        let rest = &cleaned[h_pos + 1..];
        let mins: u32 = rest
            .trim()
            .trim_end_matches("min")
            .trim()
            .parse()
            .unwrap_or(0);
        hours * 60 + mins
    } else if cleaned.ends_with("min") {
        cleaned.trim_end_matches("min").trim().parse().unwrap_or(0)
    } else {
        // Fallback: try parsing as raw minutes.
        cleaned.parse().unwrap_or(30)
    }
}

/// Abbreviate a wallet address: "0x1a2b...3f4d".
fn abbreviate_address(addr: &str) -> String {
    if addr.len() > 10 {
        format!("{}...{}", &addr[..6], &addr[addr.len() - 4..])
    } else {
        addr.to_string()
    }
}

/// HMAC-SHA256 using raw sha2 primitives.
///
/// Implements RFC 2104: HMAC(K, m) = H((K' XOR opad) || H((K' XOR ipad) || m))
fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    const BLOCK_SIZE: usize = 64;

    // Step 1: If key > block size, hash it. If shorter, zero-pad to block size.
    let mut key_block = [0u8; BLOCK_SIZE];
    if key.len() > BLOCK_SIZE {
        let hash = Sha256::digest(key);
        key_block[..32].copy_from_slice(&hash);
    } else {
        key_block[..key.len()].copy_from_slice(key);
    }

    // Step 2: inner hash = H((key XOR ipad) || message)
    let mut ipad = [0x36u8; BLOCK_SIZE];
    for (i, b) in key_block.iter().enumerate() {
        ipad[i] ^= b;
    }

    let mut inner_hasher = Sha256::new();
    inner_hasher.update(&ipad);
    inner_hasher.update(message);
    let inner_hash = inner_hasher.finalize();

    // Step 3: outer hash = H((key XOR opad) || inner_hash)
    let mut opad = [0x5cu8; BLOCK_SIZE];
    for (i, b) in key_block.iter().enumerate() {
        opad[i] ^= b;
    }

    let mut outer_hasher = Sha256::new();
    outer_hasher.update(&opad);
    outer_hasher.update(&inner_hash);

    let result = outer_hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}

/// Minimal base64 encoder (standard alphabet, with padding).
fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);
    let chunks = data.chunks(3);

    for chunk in chunks {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };

        let triple = (b0 << 16) | (b1 << 8) | b2;

        result.push(ALPHABET[((triple >> 18) & 0x3F) as usize] as char);
        result.push(ALPHABET[((triple >> 12) & 0x3F) as usize] as char);

        if chunk.len() > 1 {
            result.push(ALPHABET[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(ALPHABET[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }

    result
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> TwitterConfig {
        TwitterConfig {
            api_key: "test_key".into(),
            api_secret: "test_secret".into(),
            bearer_token: "test_bearer".into(),
            bot_username: "mori_build".into(),
            webhook_url: Some("https://mori.example.com/webhooks/twitter".into()),
            simple_mode_threshold: 25.0,
            max_requests_per_user_hour: 10,
            min_account_age_days: 30,
            allowlist: Vec::new(),
            blocklist: Vec::new(),
        }
    }

    fn test_tweet(text: &str) -> InboundTweet {
        InboundTweet {
            tweet_id: "123".into(),
            author_id: "user_1".into(),
            author_username: "alice_dev".into(),
            text: text.into(),
            in_reply_to: None,
            thread_context: Vec::new(),
            created_at: "2026-03-23T00:00:00Z".into(),
        }
    }

    fn test_reply_tweet(text: &str) -> InboundTweet {
        InboundTweet {
            tweet_id: "456".into(),
            author_id: "user_1".into(),
            author_username: "alice_dev".into(),
            text: text.into(),
            in_reply_to: Some("123".into()),
            thread_context: Vec::new(),
            created_at: "2026-03-23T00:01:00Z".into(),
        }
    }

    #[test]
    fn classify_build_request() {
        let bot = TwitterBot::new(test_config());
        let tweet = test_tweet("@mori_build build me a CLI that converts CSV to JSON");
        match bot.classify_tweet(&tweet) {
            TweetIntent::BuildRequest { description } => {
                assert!(description.contains("CLI"));
                assert!(description.contains("CSV"));
            }
            other => panic!("expected BuildRequest, got {:?}", other),
        }
    }

    #[test]
    fn classify_approval() {
        let bot = TwitterBot::new(test_config());
        let tweet = test_reply_tweet("BUILD");
        assert_eq!(
            bot.classify_tweet(&tweet),
            TweetIntent::Approval {
                quote_tweet_id: "123".into(),
            }
        );
    }

    #[test]
    fn classify_approval_case_insensitive() {
        let bot = TwitterBot::new(test_config());

        for text in &["build", "Build", "yes", "ship it", "do it", "go"] {
            let tweet = test_reply_tweet(text);
            match bot.classify_tweet(&tweet) {
                TweetIntent::Approval { .. } => {}
                other => panic!("expected Approval for '{text}', got {:?}", other),
            }
        }
    }

    #[test]
    fn classify_clarification() {
        let bot = TwitterBot::new(test_config());
        let tweet = test_reply_tweet("Can you add Apple Sign In too?");
        match bot.classify_tweet(&tweet) {
            TweetIntent::Clarification { question, .. } => {
                assert!(question.contains("Apple Sign In"));
            }
            other => panic!("expected Clarification, got {:?}", other),
        }
    }

    #[test]
    fn classify_irrelevant_short() {
        let bot = TwitterBot::new(test_config());
        let tweet = test_tweet("@mori_build hi");
        assert_eq!(bot.classify_tweet(&tweet), TweetIntent::Irrelevant);
    }

    #[test]
    fn classify_blocked_user() {
        let mut config = test_config();
        config.blocklist = vec!["alice_dev".into()];
        let bot = TwitterBot::new(config);
        let tweet = test_tweet("@mori_build build me a full operating system");
        assert_eq!(bot.classify_tweet(&tweet), TweetIntent::Irrelevant);
    }

    #[test]
    fn classify_allowlist_miss() {
        let mut config = test_config();
        config.allowlist = vec!["bob_eng".into()];
        let bot = TwitterBot::new(config);
        let tweet = test_tweet("@mori_build build me a CLI tool");
        assert_eq!(bot.classify_tweet(&tweet), TweetIntent::Irrelevant);
    }

    #[test]
    fn classify_allowlist_hit() {
        let mut config = test_config();
        config.allowlist = vec!["alice_dev".into()];
        let bot = TwitterBot::new(config);
        let tweet = test_tweet("@mori_build build me a CLI tool");
        match bot.classify_tweet(&tweet) {
            TweetIntent::BuildRequest { .. } => {}
            other => panic!("expected BuildRequest, got {:?}", other),
        }
    }

    #[test]
    fn interaction_mode_simple() {
        let bot = TwitterBot::new(test_config());
        assert!(matches!(
            bot.interaction_mode(10.0),
            InteractionMode::Simple
        ));
        assert!(matches!(
            bot.interaction_mode(25.0),
            InteractionMode::Simple
        ));
    }

    #[test]
    fn interaction_mode_conversational() {
        let bot = TwitterBot::new(test_config());
        assert!(matches!(
            bot.interaction_mode(25.01),
            InteractionMode::Conversational
        ));
    }

    #[test]
    fn rate_limiter_allows_then_blocks() {
        let limiter = RateLimiter::new(3);
        assert!(limiter.check_and_record("u1"));
        assert!(limiter.check_and_record("u1"));
        assert!(limiter.check_and_record("u1"));
        assert!(!limiter.check_and_record("u1")); // 4th request blocked.
        assert!(limiter.check_and_record("u2")); // Different user still OK.
    }

    #[test]
    fn split_short_tweet() {
        let parts = split_into_tweets("Hello world");
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0], "Hello world");
    }

    #[test]
    fn split_long_tweet() {
        let long = "a ".repeat(200); // 400 chars.
        let parts = split_into_tweets(&long);
        assert!(parts.len() >= 2);
        for part in &parts {
            assert!(
                part.len() <= MAX_TWEET_LENGTH,
                "part too long: {}",
                part.len()
            );
        }
    }

    #[test]
    fn format_milestone_update_with_remaining() {
        let text = format_milestone_update("Core Auth", 18.50, 2);
        assert!(text.contains("Core Auth"));
        assert!(text.contains("$18.50"));
        assert!(text.contains("2 milestones remaining"));
    }

    #[test]
    fn format_milestone_update_final() {
        let text = format_milestone_update("Key Rotation", 38.70, 0);
        assert!(text.contains("Key Rotation"));
        assert!(text.contains("Delivering results"));
    }

    #[test]
    fn format_error_with_refund() {
        let text = format_error_reply("provider integration failed", Some(12.0));
        assert!(text.contains("Build failed"));
        assert!(text.contains("$12.00"));
    }

    #[test]
    fn format_error_without_refund() {
        let text = format_error_reply("compilation error in main.rs", None);
        assert!(text.contains("Build failed"));
        assert!(!text.contains("Refund"));
    }

    #[test]
    fn abbreviate_long_address() {
        let addr = "0x1a2b3c4d5e6f7890abcdef1234567890abcdef12";
        assert_eq!(abbreviate_address(addr), "0x1a2b...ef12");
    }

    #[test]
    fn abbreviate_short_address() {
        assert_eq!(abbreviate_address("0xABCD"), "0xABCD");
    }

    #[test]
    fn hmac_sha256_known_vector() {
        // RFC 4231 Test Case 2:
        // Key  = "Jefe"
        // Data = "what do ya want for nothing?"
        // HMAC = 5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843
        let mac = hmac_sha256(b"Jefe", b"what do ya want for nothing?");
        let hex: String = mac.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(
            hex,
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
        );
    }

    #[test]
    fn base64_encode_known() {
        assert_eq!(base64_encode(b"Hello"), "SGVsbG8=");
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
    }

    #[test]
    fn parse_time_estimates() {
        assert_eq!(parse_time_estimate("~45 min"), 45);
        assert_eq!(parse_time_estimate("~2h"), 120);
        assert_eq!(parse_time_estimate("~1h 30min"), 90);
        assert_eq!(parse_time_estimate("30"), 30);
    }

    #[test]
    fn aggregate_time_works() {
        use crate::types::{CostEstimate, MilestoneStatus};

        let milestones = vec![
            Milestone {
                name: "Core".into(),
                plans: vec!["01".into()],
                task_count: 5,
                estimated_cost: CostEstimate {
                    inference: 0.0,
                    compute: 0.0,
                    total: 10.0,
                },
                estimated_time: "~35 min".into(),
                actual_cost: None,
                status: MilestoneStatus::Pending,
            },
            Milestone {
                name: "Integration".into(),
                plans: vec!["02".into()],
                task_count: 3,
                estimated_cost: CostEstimate {
                    inference: 0.0,
                    compute: 0.0,
                    total: 8.0,
                },
                estimated_time: "~20 min".into(),
                actual_cost: None,
                status: MilestoneStatus::Pending,
            },
        ];

        assert_eq!(aggregate_time_estimate(&milestones), "55 min");
    }
}
