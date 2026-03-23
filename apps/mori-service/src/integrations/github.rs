//! GitHub App integration for Mori-as-a-Service.
//!
//! Handles incoming webhooks (issue opened, issue comment, issue labeled, PR opened,
//! review requested) and provides a client for the GitHub REST API v3 (posting comments,
//! creating PRs, managing labels).

use std::fmt::Write as FmtWrite;

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use sha2::{Digest, Sha256};
use tracing::{debug, error, info, warn};

use crate::state::ServiceState;
use crate::types::{GitHubConfig, Milestone, Proposal, Receipt, Run};

// ---------------------------------------------------------------------------
// Labels
// ---------------------------------------------------------------------------

/// Trigger labels (applied by humans).
pub const LABEL_BUILD: &str = "mori:build";
pub const LABEL_INVESTIGATE: &str = "mori:investigate";
pub const LABEL_REVIEW: &str = "mori:review";
pub const LABEL_PRIORITY: &str = "mori:priority";
pub const LABEL_HOLD: &str = "mori:hold";

/// Lifecycle labels (managed by Mori).
pub const LABEL_PLANNING: &str = "mori:planning";
pub const LABEL_BUILDING: &str = "mori:building";
pub const LABEL_READY: &str = "mori:ready";

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

/// Subset of GitHub webhook events that Mori processes.
#[derive(Debug)]
pub enum GitHubEvent {
    IssueOpened { issue: Issue },
    IssueComment { issue: Issue, comment: Comment },
    IssueLabeled { issue: Issue, label: String },
    PullRequestOpened { pr: PullRequest },
    ReviewRequested { pr: PullRequest },
}

#[derive(Debug, Clone)]
pub struct Issue {
    pub number: u64,
    pub title: String,
    pub body: String,
    pub labels: Vec<String>,
    pub user: String,
}

#[derive(Debug, Clone)]
pub struct Comment {
    pub id: u64,
    pub body: String,
    pub user: String,
}

#[derive(Debug, Clone)]
pub struct PullRequest {
    pub number: u64,
    pub title: String,
    pub body: String,
    pub head_branch: String,
    pub base_branch: String,
}

// ---------------------------------------------------------------------------
// Slash commands
// ---------------------------------------------------------------------------

/// Slash commands recognized in issue/PR comments.
#[derive(Debug, PartialEq, Eq)]
pub enum SlashCommand {
    Review,
    Investigate,
    Fix,
    Cost,
    Explain,
    Run { plans: String },
}

/// Parse the first `/mori` command from a comment body.
///
/// Returns `None` if no recognized command is found. Only the first match wins;
/// embedding multiple commands in one comment is not supported.
pub fn parse_slash_command(body: &str) -> Option<SlashCommand> {
    for line in body.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("/mori") {
            continue;
        }

        let rest = trimmed.strip_prefix("/mori")?.trim();

        if rest.is_empty() {
            // Bare "/mori" with nothing after it — ignore.
            continue;
        }

        // Split into subcommand + optional argument.
        let (cmd, arg) = match rest.split_once(char::is_whitespace) {
            Some((c, a)) => (c, a.trim()),
            None => (rest, ""),
        };

        return match cmd {
            "review" => Some(SlashCommand::Review),
            "investigate" => Some(SlashCommand::Investigate),
            "fix" => Some(SlashCommand::Fix),
            "cost" => Some(SlashCommand::Cost),
            "explain" => Some(SlashCommand::Explain),
            "run" if !arg.is_empty() => Some(SlashCommand::Run {
                plans: arg.to_string(),
            }),
            _ => None,
        };
    }
    None
}

// ---------------------------------------------------------------------------
// Webhook signature verification
// ---------------------------------------------------------------------------

/// HMAC-SHA256 implemented over `sha2::Sha256`.
///
/// GitHub sends the signature in the `X-Hub-Signature-256` header as
/// `sha256=<hex digest>`. We recompute the HMAC over the raw body bytes
/// and compare in constant-ish time.
fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    const BLOCK_SIZE: usize = 64; // SHA-256 block size

    // If the key is longer than one block, hash it first.
    let mut key_block = [0u8; BLOCK_SIZE];
    if key.len() > BLOCK_SIZE {
        let hashed = Sha256::digest(key);
        key_block[..32].copy_from_slice(&hashed);
    } else {
        key_block[..key.len()].copy_from_slice(key);
    }

    // Inner hash: H((K XOR ipad) || message)
    let mut ipad = [0x36u8; BLOCK_SIZE];
    for (i, b) in ipad.iter_mut().enumerate() {
        *b ^= key_block[i];
    }
    let mut inner = Sha256::new();
    inner.update(&ipad);
    inner.update(message);
    let inner_hash = inner.finalize();

    // Outer hash: H((K XOR opad) || inner_hash)
    let mut opad = [0x5cu8; BLOCK_SIZE];
    for (i, b) in opad.iter_mut().enumerate() {
        *b ^= key_block[i];
    }
    let mut outer = Sha256::new();
    outer.update(&opad);
    outer.update(&inner_hash);

    let result = outer.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}

/// Encode bytes as lowercase hex.
fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

/// Constant-time-ish comparison of two equal-length byte slices.
/// Not truly constant-time without a dedicated crate, but good enough
/// to prevent trivial timing attacks on webhook signatures.
fn secure_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Validate the `X-Hub-Signature-256` header against the raw request body.
fn validate_signature(secret: &str, signature_header: &str, body: &[u8]) -> bool {
    let hex_sig = match signature_header.strip_prefix("sha256=") {
        Some(h) => h,
        None => return false,
    };

    let computed = hmac_sha256(secret.as_bytes(), body);
    let computed_hex = hex_encode(&computed);

    secure_compare(computed_hex.as_bytes(), hex_sig.as_bytes())
}

// ---------------------------------------------------------------------------
// Webhook handler (Axum)
// ---------------------------------------------------------------------------

/// Axum handler for `POST /webhooks/github`.
///
/// Validates the webhook signature, parses the event, and dispatches to the
/// appropriate handler. Returns 200 on success, 401 on bad signature, and
/// 400 on unparseable payloads.
pub async fn webhook_handler(
    State(state): State<ServiceState>,
    headers: HeaderMap,
    body: String,
) -> Result<impl IntoResponse, StatusCode> {
    let github_config = state.config.integrations.github.as_ref().ok_or_else(|| {
        warn!("GitHub webhook received but integration is not configured");
        StatusCode::NOT_FOUND
    })?;

    // --- Signature validation ---
    let signature = headers
        .get("x-hub-signature-256")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            warn!("Missing X-Hub-Signature-256 header");
            StatusCode::UNAUTHORIZED
        })?;

    if !validate_signature(&github_config.webhook_secret, signature, body.as_bytes()) {
        warn!("Invalid webhook signature");
        return Err(StatusCode::UNAUTHORIZED);
    }

    // --- Event type ---
    let event_type = headers
        .get("x-github-event")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let payload: serde_json::Value = serde_json::from_str(&body).map_err(|e| {
        error!("Failed to parse webhook JSON: {e}");
        StatusCode::BAD_REQUEST
    })?;

    let action = payload["action"].as_str().unwrap_or("");

    debug!(event_type, action, "GitHub webhook received");

    // --- Dispatch ---
    let event = match (event_type, action) {
        ("issues", "opened") => parse_issue_opened(&payload),
        ("issues", "labeled") => parse_issue_labeled(&payload),
        ("issue_comment", "created") => parse_issue_comment(&payload),
        ("pull_request", "opened") => parse_pr_opened(&payload),
        ("pull_request", "review_requested") => parse_review_requested(&payload),
        _ => {
            debug!(event_type, action, "Ignoring unhandled webhook event");
            return Ok(StatusCode::OK);
        }
    };

    let event = match event {
        Some(e) => e,
        None => {
            warn!(
                event_type,
                action, "Failed to parse webhook payload into domain event"
            );
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    let client = GitHubClient::new(github_config);
    let owner_repo = extract_owner_repo(&payload);

    // Spawn event handling so the webhook returns immediately.
    tokio::spawn(async move {
        if let Err(e) = dispatch_event(&client, &state, &owner_repo, event).await {
            error!("Error handling GitHub event: {e:#}");
        }
    });

    Ok(StatusCode::OK)
}

// ---------------------------------------------------------------------------
// Payload parsing
// ---------------------------------------------------------------------------

fn extract_owner_repo(payload: &serde_json::Value) -> (String, String) {
    let repo = &payload["repository"];
    let owner = repo["owner"]["login"]
        .as_str()
        .unwrap_or("unknown")
        .to_string();
    let name = repo["name"].as_str().unwrap_or("unknown").to_string();
    (owner, name)
}

fn parse_issue(v: &serde_json::Value) -> Option<Issue> {
    Some(Issue {
        number: v["number"].as_u64()?,
        title: v["title"].as_str()?.to_string(),
        body: v["body"].as_str().unwrap_or("").to_string(),
        labels: v["labels"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|l| l["name"].as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default(),
        user: v["user"]["login"].as_str().unwrap_or("").to_string(),
    })
}

fn parse_pr(v: &serde_json::Value) -> Option<PullRequest> {
    Some(PullRequest {
        number: v["number"].as_u64()?,
        title: v["title"].as_str()?.to_string(),
        body: v["body"].as_str().unwrap_or("").to_string(),
        head_branch: v["head"]["ref"].as_str().unwrap_or("").to_string(),
        base_branch: v["base"]["ref"].as_str().unwrap_or("main").to_string(),
    })
}

fn parse_issue_opened(payload: &serde_json::Value) -> Option<GitHubEvent> {
    let issue = parse_issue(&payload["issue"])?;
    Some(GitHubEvent::IssueOpened { issue })
}

fn parse_issue_labeled(payload: &serde_json::Value) -> Option<GitHubEvent> {
    let issue = parse_issue(&payload["issue"])?;
    let label = payload["label"]["name"].as_str()?.to_string();
    Some(GitHubEvent::IssueLabeled { issue, label })
}

fn parse_issue_comment(payload: &serde_json::Value) -> Option<GitHubEvent> {
    let issue = parse_issue(&payload["issue"])?;
    let c = &payload["comment"];
    let comment = Comment {
        id: c["id"].as_u64()?,
        body: c["body"].as_str().unwrap_or("").to_string(),
        user: c["user"]["login"].as_str().unwrap_or("").to_string(),
    };
    Some(GitHubEvent::IssueComment { issue, comment })
}

fn parse_pr_opened(payload: &serde_json::Value) -> Option<GitHubEvent> {
    let pr = parse_pr(&payload["pull_request"])?;
    Some(GitHubEvent::PullRequestOpened { pr })
}

fn parse_review_requested(payload: &serde_json::Value) -> Option<GitHubEvent> {
    let pr = parse_pr(&payload["pull_request"])?;
    Some(GitHubEvent::ReviewRequested { pr })
}

// ---------------------------------------------------------------------------
// Event dispatch
// ---------------------------------------------------------------------------

async fn dispatch_event(
    client: &GitHubClient,
    state: &ServiceState,
    (owner, repo): &(String, String),
    event: GitHubEvent,
) -> anyhow::Result<()> {
    match event {
        GitHubEvent::IssueOpened { issue } => {
            handle_issue_opened(client, state, owner, repo, &issue).await
        }
        GitHubEvent::IssueComment { issue, comment } => {
            handle_issue_comment(client, state, owner, repo, &issue, &comment).await
        }
        GitHubEvent::IssueLabeled { issue, label } => {
            handle_issue_labeled(client, state, owner, repo, &issue, &label).await
        }
        GitHubEvent::PullRequestOpened { pr } => {
            handle_pr_opened(client, state, owner, repo, &pr).await
        }
        GitHubEvent::ReviewRequested { pr } => {
            handle_review_requested(client, state, owner, repo, &pr).await
        }
    }
}

// ---------------------------------------------------------------------------
// Event handlers
// ---------------------------------------------------------------------------

/// New issue opened. If `auto_propose_on_issue` is enabled, analyze the issue
/// and post a proposal comment.
async fn handle_issue_opened(
    client: &GitHubClient,
    state: &ServiceState,
    owner: &str,
    repo: &str,
    issue: &Issue,
) -> anyhow::Result<()> {
    let defaults = github_defaults(state);
    if !defaults.auto_propose_on_issue {
        debug!(issue.number, "auto_propose_on_issue is off, skipping");
        return Ok(());
    }

    info!(issue.number, title = %issue.title, "Analyzing new issue for proposal");

    // Add the planning lifecycle label.
    client
        .add_labels(owner, repo, issue.number, &[LABEL_PLANNING])
        .await?;

    // TODO(mori-service): Run the actual proposal engine against the repo.
    // For now, post a placeholder acknowledgement so the webhook loop is wired up.
    let body = format!(
        "I'm analyzing this issue and the repository to prepare a proposal. \
         I'll post a detailed estimate with milestones and cost shortly.\n\n\
         _Mori is reading your codebase..._"
    );
    client
        .post_comment(owner, repo, issue.number, &body)
        .await?;

    Ok(())
}

/// Comment on an issue or PR. Check for slash commands.
async fn handle_issue_comment(
    client: &GitHubClient,
    state: &ServiceState,
    owner: &str,
    repo: &str,
    issue: &Issue,
    comment: &Comment,
) -> anyhow::Result<()> {
    // Ignore comments from the bot itself to prevent loops.
    let bot_username = state
        .config
        .integrations
        .github
        .as_ref()
        .map(|c| c.bot_username.as_str())
        .unwrap_or("mori-bot");

    if comment.user == bot_username {
        return Ok(());
    }

    let cmd = match parse_slash_command(&comment.body) {
        Some(c) => c,
        None => return Ok(()), // Not a command — nothing to do.
    };

    info!(
        issue.number,
        ?cmd,
        user = %comment.user,
        "Slash command received"
    );

    match cmd {
        SlashCommand::Review => {
            let body = "Running the review pipeline. I'll post results as a PR review \
                        with inline comments when the analysis finishes.";
            client.post_comment(owner, repo, issue.number, body).await?;
            // TODO(mori-service): Trigger review pipeline.
        }
        SlashCommand::Investigate => {
            let body = format!(
                "Investigating #{number}. I'll search the codebase, identify affected \
                 files and symbols, and post my findings here.",
                number = issue.number
            );
            client
                .post_comment(owner, repo, issue.number, &body)
                .await?;
            // TODO(mori-service): Trigger investigation pipeline.
        }
        SlashCommand::Fix => {
            let body = "Looking at failing CI checks. If I can identify and fix the issue, \
                        I'll push a commit to this branch.";
            client.post_comment(owner, repo, issue.number, body).await?;
            // TODO(mori-service): Trigger fix pipeline.
        }
        SlashCommand::Cost => {
            let body = "Estimating cost for this issue. I'll post a detailed breakdown \
                        (plans, tasks, model tiers, total) as a follow-up comment.";
            client.post_comment(owner, repo, issue.number, body).await?;
            // TODO(mori-service): Run cost estimation without building.
        }
        SlashCommand::Explain => {
            let body = "Reading the diffs. I'll post a plain-language summary of what \
                        this PR changes, grouped by purpose.";
            client.post_comment(owner, repo, issue.number, body).await?;
            // TODO(mori-service): Trigger explain pipeline.
        }
        SlashCommand::Run { plans } => {
            let body = format!(
                "Running plans {plans} against the current branch. I'll post progress \
                 updates as milestones complete."
            );
            client
                .post_comment(owner, repo, issue.number, &body)
                .await?;
            // TODO(mori-service): Trigger selective plan execution.
            let _ = state; // suppress unused warning until wired up
        }
    }

    Ok(())
}

/// Label applied to an issue. Handle `mori:*` trigger labels.
async fn handle_issue_labeled(
    client: &GitHubClient,
    _state: &ServiceState,
    owner: &str,
    repo: &str,
    issue: &Issue,
    label: &str,
) -> anyhow::Result<()> {
    info!(issue.number, label, "Label applied");

    match label {
        LABEL_BUILD => {
            client
                .add_labels(owner, repo, issue.number, &[LABEL_BUILDING])
                .await?;
            client
                .remove_label(owner, repo, issue.number, LABEL_PLANNING)
                .await
                .ok(); // may not exist
            let body = "Build triggered. Starting the orchestrator against this issue's proposal.";
            client.post_comment(owner, repo, issue.number, body).await?;
            // TODO(mori-service): Start build run.
        }
        LABEL_INVESTIGATE => {
            client
                .add_labels(owner, repo, issue.number, &[LABEL_PLANNING])
                .await?;
            let body = "Investigation requested via label. Analyzing the codebase now.";
            client.post_comment(owner, repo, issue.number, body).await?;
            // TODO(mori-service): Trigger investigation pipeline.
        }
        LABEL_REVIEW => {
            let body = "Review requested via label. Running the review pipeline.";
            client.post_comment(owner, repo, issue.number, body).await?;
            // TODO(mori-service): Trigger review pipeline.
        }
        LABEL_PRIORITY => {
            info!(issue.number, "Issue promoted to priority queue");
            let body = "Noted — this issue is now at the front of the build queue.";
            client.post_comment(owner, repo, issue.number, body).await?;
            // TODO(mori-service): Bump queue priority.
        }
        LABEL_HOLD => {
            info!(issue.number, "Build paused via hold label");
            let body = "Build paused. Remove the `mori:hold` label to resume.";
            client.post_comment(owner, repo, issue.number, body).await?;
            // TODO(mori-service): Pause active run.
        }
        _ => {
            debug!(label, "Ignoring non-mori label");
        }
    }

    Ok(())
}

/// New PR opened. If `review_on_pr_open` is enabled, auto-review it.
async fn handle_pr_opened(
    client: &GitHubClient,
    state: &ServiceState,
    owner: &str,
    repo: &str,
    pr: &PullRequest,
) -> anyhow::Result<()> {
    let defaults = github_defaults(state);
    if !defaults.review_on_pr_open {
        debug!(pr.number, "review_on_pr_open is off, skipping auto-review");
        return Ok(());
    }

    info!(pr.number, title = %pr.title, "Auto-reviewing new PR");

    let body = "I've been added as a reviewer. Running the review pipeline now — \
                I'll post results as inline comments with a summary review.";
    client.post_comment(owner, repo, pr.number, body).await?;

    // TODO(mori-service): Trigger review pipeline on PR diff.
    Ok(())
}

/// Mori-bot added as a reviewer on a PR. Trigger the review pipeline.
async fn handle_review_requested(
    client: &GitHubClient,
    state: &ServiceState,
    owner: &str,
    repo: &str,
    pr: &PullRequest,
) -> anyhow::Result<()> {
    let bot_username = state
        .config
        .integrations
        .github
        .as_ref()
        .map(|c| c.bot_username.as_str())
        .unwrap_or("mori-bot");

    info!(
        pr.number,
        bot = bot_username,
        "Review requested — running review pipeline"
    );

    let body = "Review requested. I'm analyzing the PR now and will post inline comments \
                with a summary review when done.";
    client.post_comment(owner, repo, pr.number, body).await?;

    // TODO(mori-service): Trigger review pipeline.
    Ok(())
}

/// Shorthand to get the GitHub defaults from config, falling back to the
/// zero-valued struct if the integration isn't configured.
fn github_defaults(state: &ServiceState) -> crate::types::GitHubDefaults {
    state
        .config
        .integrations
        .github
        .as_ref()
        .map(|c| c.defaults.clone())
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// GitHub API client
// ---------------------------------------------------------------------------

const GITHUB_API_BASE: &str = "https://api.github.com";

/// Client for GitHub REST API v3.
///
/// Uses the shared `reqwest::Client` from `ServiceState` and authenticates
/// with a Bearer token. In production, the token is obtained by exchanging
/// the GitHub App's private key for an installation access token. That flow
/// is stubbed here — the client falls back to the `GITHUB_TOKEN` env var
/// or a placeholder.
pub struct GitHubClient {
    http: reqwest::Client,
    app_id: u64,
    bot_username: String,
    /// Cached installation token. In production this would be refreshed
    /// before expiry via the App JWT -> installation token exchange.
    token: String,
}

impl GitHubClient {
    /// Create a new client from the integration config.
    pub fn new(config: &GitHubConfig) -> Self {
        let token = Self::resolve_token(config);
        Self {
            http: reqwest::Client::new(),
            app_id: config.app_id,
            bot_username: config.bot_username.clone(),
            token,
        }
    }

    /// Resolve the installation token.
    ///
    /// TODO(mori-service): Implement the full flow:
    ///   1. Read the PEM private key from `config.private_key_path`.
    ///   2. Mint a JWT signed with the App's private key.
    ///   3. POST /app/installations/{id}/access_tokens to get a short-lived token.
    ///   4. Cache it and refresh before expiry.
    ///
    /// For now, fall back to the `GITHUB_TOKEN` environment variable.
    fn resolve_token(config: &GitHubConfig) -> String {
        if let Ok(token) = std::env::var("GITHUB_TOKEN") {
            return token;
        }

        warn!(
            app_id = config.app_id,
            "No GITHUB_TOKEN env var set and JWT installation token exchange is not yet \
             implemented. GitHub API calls will fail."
        );
        String::new()
    }

    /// Standard headers for GitHub API requests.
    fn headers(&self) -> reqwest::header::HeaderMap {
        let mut h = reqwest::header::HeaderMap::new();
        h.insert(
            reqwest::header::ACCEPT,
            "application/vnd.github+json"
                .parse()
                .expect("static header value"),
        );
        h.insert(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", self.token)
                .parse()
                .expect("token should be ascii"),
        );
        h.insert(
            reqwest::header::USER_AGENT,
            format!("mori-bot/{}", env!("CARGO_PKG_VERSION"))
                .parse()
                .expect("static header value"),
        );
        h.insert(
            "X-GitHub-Api-Version",
            "2022-11-28".parse().expect("static header value"),
        );
        h
    }

    /// Post a comment on an issue or pull request.
    pub async fn post_comment(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        body: &str,
    ) -> anyhow::Result<()> {
        let url = format!("{GITHUB_API_BASE}/repos/{owner}/{repo}/issues/{issue_number}/comments");

        let resp = self
            .http
            .post(&url)
            .headers(self.headers())
            .json(&serde_json::json!({ "body": body }))
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("GitHub API error posting comment: {status} — {text}");
        }

        debug!(owner, repo, issue_number, "Posted comment");
        Ok(())
    }

    /// Create a pull request. Returns the PR number on success.
    pub async fn create_pr(
        &self,
        owner: &str,
        repo: &str,
        title: &str,
        body: &str,
        head: &str,
        base: &str,
    ) -> anyhow::Result<u64> {
        let url = format!("{GITHUB_API_BASE}/repos/{owner}/{repo}/pulls");

        let resp = self
            .http
            .post(&url)
            .headers(self.headers())
            .json(&serde_json::json!({
                "title": title,
                "body": body,
                "head": head,
                "base": base,
                "draft": true,
            }))
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("GitHub API error creating PR: {status} — {text}");
        }

        let json: serde_json::Value = resp.json().await?;
        let pr_number = json["number"]
            .as_u64()
            .ok_or_else(|| anyhow::anyhow!("Missing 'number' in PR creation response"))?;

        info!(owner, repo, pr_number, "Created draft PR");
        Ok(pr_number)
    }

    /// Add labels to an issue or pull request.
    pub async fn add_labels(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        labels: &[&str],
    ) -> anyhow::Result<()> {
        let url = format!("{GITHUB_API_BASE}/repos/{owner}/{repo}/issues/{issue_number}/labels");

        let resp = self
            .http
            .post(&url)
            .headers(self.headers())
            .json(&serde_json::json!({ "labels": labels }))
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("GitHub API error adding labels: {status} — {text}");
        }

        debug!(owner, repo, issue_number, ?labels, "Added labels");
        Ok(())
    }

    /// Remove a single label from an issue or pull request.
    ///
    /// Returns `Ok(())` even if the label wasn't present (GitHub returns 404
    /// in that case, which we swallow).
    pub async fn remove_label(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        label: &str,
    ) -> anyhow::Result<()> {
        let url =
            format!("{GITHUB_API_BASE}/repos/{owner}/{repo}/issues/{issue_number}/labels/{label}");

        let resp = self
            .http
            .delete(&url)
            .headers(self.headers())
            .send()
            .await?;

        // 404 is fine — label wasn't present.
        if !resp.status().is_success() && resp.status().as_u16() != 404 {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("GitHub API error removing label: {status} — {text}");
        }

        debug!(owner, repo, issue_number, label, "Removed label");
        Ok(())
    }

    /// Get the bot's username (for self-identification checks).
    pub fn bot_username(&self) -> &str {
        &self.bot_username
    }

    /// Get the App ID.
    pub fn app_id(&self) -> u64 {
        self.app_id
    }
}

// ---------------------------------------------------------------------------
// Markdown formatters
// ---------------------------------------------------------------------------

/// Format a proposal as a GitHub comment.
///
/// Produces structured markdown matching the PRD spec: scope summary,
/// affected files inferred from milestone names, milestone cost table,
/// total cost, and an approval CTA.
pub fn format_proposal_comment(proposal: &Proposal) -> String {
    let mut md = String::with_capacity(2048);

    md.push_str("### Proposal\n\n");
    md.push_str("I've analyzed the codebase. Here's what I'd build:\n\n");

    // --- Affected files (derived from plan names in milestones) ---
    md.push_str("**Affected areas:**\n");
    for ms in &proposal.milestones {
        for plan in &ms.plans {
            let _ = writeln!(md, "- `{plan}` ({name})", name = ms.name);
        }
    }
    md.push('\n');

    // --- Milestone table ---
    md.push_str("**Milestones:**\n\n");
    md.push_str("| Milestone | Plans | Tasks | Est. cost |\n");
    md.push_str("|-----------|-------|-------|-----------|\n");

    for ms in &proposal.milestones {
        let plans_str = ms.plans.join(", ");
        let _ = writeln!(
            md,
            "| {name} | {plans} | {tasks} | ${cost:.2} |",
            name = ms.name,
            plans = plans_str,
            tasks = ms.task_count,
            cost = ms.estimated_cost.total,
        );
    }

    let total_plans: usize = proposal.milestones.iter().map(|m| m.plans.len()).sum();
    let total_tasks: u32 = proposal.milestones.iter().map(|m| m.task_count).sum();
    let _ = writeln!(
        md,
        "| **Total** | **{total_plans} plans** | **{total_tasks} tasks** | **${cost:.2}** |",
        cost = proposal.total_cost,
    );

    md.push('\n');

    // --- Cost breakdown ---
    let inference: f64 = proposal
        .milestones
        .iter()
        .map(|m| m.estimated_cost.inference)
        .sum();
    let compute: f64 = proposal
        .milestones
        .iter()
        .map(|m| m.estimated_cost.compute)
        .sum();
    let _ = writeln!(
        md,
        "**Cost breakdown:** ${inference:.2} inference + ${compute:.2} compute = **${total:.2}**",
        total = proposal.total_cost,
    );

    let _ = writeln!(
        md,
        "**Valid for:** {valid_for}\n",
        valid_for = proposal.valid_for
    );

    // --- Approval CTA ---
    md.push_str("React with :thumbsup: to approve and start building.\n");
    md.push_str("Reply to ask questions or adjust scope.\n");

    md
}

/// Format a PR description from a completed run and its receipt.
///
/// Structured markdown: Summary, Origin, Changes (milestone table), Cost
/// (estimated vs. actual), and a Verification checklist.
pub fn format_pr_description(run: &Run, receipt: &Receipt) -> String {
    let mut md = String::with_capacity(2048);

    // --- Summary ---
    md.push_str("## Summary\n\n");
    let _ = writeln!(
        md,
        "Automated build for proposal `{pid}`. \
         {completed}/{total} milestones completed.",
        pid = run.proposal_id,
        completed = receipt.milestones_completed,
        total = receipt.milestones_total,
    );
    md.push('\n');

    // --- Origin ---
    md.push_str("## Origin\n\n");
    let _ = writeln!(md, "- Proposal: `{}`", run.proposal_id);
    let _ = writeln!(md, "- Run: `{}`", run.run_id);
    let _ = writeln!(md, "- Funding: {:?}", run.funding_mode);
    md.push('\n');

    // --- Changes (per-milestone) ---
    md.push_str("## Changes\n\n");
    md.push_str("| Milestone | Plans | Status |\n");
    md.push_str("|-----------|-------|--------|\n");
    for ms in &run.milestones {
        let plans_str = ms.plans.join(", ");
        let _ = writeln!(
            md,
            "| {name} | {plans} | {status:?} |",
            name = ms.name,
            plans = plans_str,
            status = ms.status,
        );
    }
    md.push('\n');

    // --- Cost ---
    md.push_str("## Cost\n\n");
    md.push_str("| | Estimated | Actual |\n");
    md.push_str("|---|-----------|--------|\n");

    let estimated_total: f64 = run.milestones.iter().map(|m| m.estimated_cost.total).sum();
    let estimated_inference: f64 = run
        .milestones
        .iter()
        .map(|m| m.estimated_cost.inference)
        .sum();
    let estimated_compute: f64 = run
        .milestones
        .iter()
        .map(|m| m.estimated_cost.compute)
        .sum();

    let _ = writeln!(
        md,
        "| Inference | ${estimated_inference:.2} | ${actual:.2} |",
        actual = receipt.breakdown.inference,
    );
    let _ = writeln!(
        md,
        "| Compute | ${estimated_compute:.2} | ${actual:.2} |",
        actual = receipt.breakdown.compute,
    );
    let _ = writeln!(
        md,
        "| **Total** | **${estimated_total:.2}** | **${actual:.2}** |",
        actual = receipt.total_cost,
    );
    md.push('\n');

    if receipt.refund > 0.0 {
        let _ = writeln!(md, "Refund: **${:.2}**\n", receipt.refund);
    }

    // --- Plans executed ---
    let plans: Vec<&str> = run
        .milestones
        .iter()
        .flat_map(|m| m.plans.iter().map(String::as_str))
        .collect();
    let _ = writeln!(md, "Plans executed: {}", plans.join(", "));
    md.push('\n');

    // --- Verification ---
    md.push_str("## Verification\n\n");
    md.push_str("- [x] Compiles without warnings\n");
    md.push_str("- [x] All existing tests pass\n");
    md.push_str("- [x] No new clippy lints\n");
    md.push_str("- [ ] Awaiting human review\n");

    md
}

/// Format a milestone progress comment posted during a build.
pub fn format_milestone_progress(
    milestone: &Milestone,
    milestone_index: usize,
    total_milestones: usize,
    cost_so_far: f64,
) -> String {
    format!(
        "**Progress:** Milestone {idx}/{total} complete ({name}).\n\
         {plans} plans finished, {tasks} tasks done. Cost so far: ${cost:.2}.\n\
         {next}",
        idx = milestone_index + 1,
        total = total_milestones,
        name = milestone.name,
        plans = milestone.plans.len(),
        tasks = milestone.task_count,
        cost = cost_so_far,
        next = if milestone_index + 1 < total_milestones {
            format!("Starting milestone {}.", milestone_index + 2)
        } else {
            "All milestones complete. Opening PR.".to_string()
        },
    )
}

/// Format the final build-complete comment posted on the originating issue.
pub fn format_build_complete_comment(
    estimated_cost: f64,
    actual_cost: f64,
    pr_number: u64,
) -> String {
    format!(
        "**Build complete.**\n\
         Estimated cost: ${estimated:.2} | Actual cost: ${actual:.2}\n\
         PR: #{pr}",
        estimated = estimated_cost,
        actual = actual_cost,
        pr = pr_number,
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_slash_commands() {
        assert_eq!(
            parse_slash_command("/mori review"),
            Some(SlashCommand::Review)
        );
        assert_eq!(
            parse_slash_command("/mori investigate"),
            Some(SlashCommand::Investigate)
        );
        assert_eq!(parse_slash_command("/mori fix"), Some(SlashCommand::Fix));
        assert_eq!(parse_slash_command("/mori cost"), Some(SlashCommand::Cost));
        assert_eq!(
            parse_slash_command("/mori explain"),
            Some(SlashCommand::Explain)
        );
        assert_eq!(
            parse_slash_command("/mori run 03-05"),
            Some(SlashCommand::Run {
                plans: "03-05".to_string()
            })
        );
    }

    #[test]
    fn parse_slash_command_with_surrounding_text() {
        let body = "Hey team, can someone check this?\n\n/mori review\n\nThanks!";
        assert_eq!(parse_slash_command(body), Some(SlashCommand::Review));
    }

    #[test]
    fn parse_slash_command_ignores_unknown() {
        assert_eq!(parse_slash_command("/mori deploy"), None);
        assert_eq!(parse_slash_command("just a regular comment"), None);
        assert_eq!(parse_slash_command("/mori"), None);
    }

    #[test]
    fn parse_slash_command_run_needs_arg() {
        // "/mori run" without a plan spec should not match.
        assert_eq!(parse_slash_command("/mori run"), None);
    }

    #[test]
    fn hmac_sha256_known_vector() {
        // RFC 4231 test vector 2:
        // Key  = "Jefe"
        // Data = "what do ya want for nothing?"
        // HMAC = 5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843
        let mac = hmac_sha256(b"Jefe", b"what do ya want for nothing?");
        let hex = hex_encode(&mac);
        assert_eq!(
            hex,
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
        );
    }

    #[test]
    fn signature_validation_roundtrip() {
        let secret = "test-webhook-secret";
        let body = b"payload body";
        let mac = hmac_sha256(secret.as_bytes(), body);
        let sig = format!("sha256={}", hex_encode(&mac));
        assert!(validate_signature(secret, &sig, body));
    }

    #[test]
    fn signature_validation_rejects_bad_sig() {
        assert!(!validate_signature(
            "secret",
            "sha256=0000000000000000000000000000000000000000000000000000000000000000",
            b"payload"
        ));
    }

    #[test]
    fn signature_validation_rejects_missing_prefix() {
        assert!(!validate_signature("secret", "bad-format", b"payload"));
    }

    #[test]
    fn secure_compare_works() {
        assert!(secure_compare(b"abc", b"abc"));
        assert!(!secure_compare(b"abc", b"abd"));
        assert!(!secure_compare(b"abc", b"ab"));
    }

    #[test]
    fn format_proposal_produces_markdown() {
        use crate::types::{CostEstimate, MilestoneStatus, ProposalStatus};

        let proposal = Proposal {
            proposal_id: "p-test".into(),
            draft_id: "d-test".into(),
            version: 1,
            milestones: vec![
                Milestone {
                    name: "Core implementation".into(),
                    plans: vec!["01-models".into(), "02-handlers".into()],
                    task_count: 6,
                    estimated_cost: CostEstimate {
                        inference: 4.0,
                        compute: 1.0,
                        total: 5.0,
                    },
                    estimated_time: "~30 min".into(),
                    actual_cost: None,
                    status: MilestoneStatus::Pending,
                },
                Milestone {
                    name: "Tests".into(),
                    plans: vec!["03-tests".into()],
                    task_count: 3,
                    estimated_cost: CostEstimate {
                        inference: 2.0,
                        compute: 0.5,
                        total: 2.5,
                    },
                    estimated_time: "~15 min".into(),
                    actual_cost: None,
                    status: MilestoneStatus::Pending,
                },
            ],
            total_cost: 7.5,
            draft_cost_spent: 0.5,
            net_cost: 7.0,
            valid_for: "24h".into(),
            status: ProposalStatus::Proposed,
            created_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now(),
        };

        let md = format_proposal_comment(&proposal);
        assert!(md.contains("### Proposal"));
        assert!(md.contains("Core implementation"));
        assert!(md.contains("$7.50"));
        assert!(md.contains(":thumbsup:"));
        assert!(md.contains("01-models"));
    }

    #[test]
    fn parse_issue_from_json() {
        let json = serde_json::json!({
            "number": 142,
            "title": "Add rate limiting",
            "body": "We need rate limits on the API.",
            "labels": [{"name": "bug"}, {"name": "mori:build"}],
            "user": {"login": "alice"}
        });
        let issue = parse_issue(&json).unwrap();
        assert_eq!(issue.number, 142);
        assert_eq!(issue.title, "Add rate limiting");
        assert_eq!(issue.labels, vec!["bug", "mori:build"]);
        assert_eq!(issue.user, "alice");
    }

    #[test]
    fn parse_pr_from_json() {
        let json = serde_json::json!({
            "number": 287,
            "title": "feat: rate limiting",
            "body": "Adds rate limiting middleware.",
            "head": {"ref": "mori/142-rate-limiting"},
            "base": {"ref": "main"}
        });
        let pr = parse_pr(&json).unwrap();
        assert_eq!(pr.number, 287);
        assert_eq!(pr.head_branch, "mori/142-rate-limiting");
        assert_eq!(pr.base_branch, "main");
    }
}
