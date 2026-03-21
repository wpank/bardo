//! `golem.toml` runtime configuration schema.

use std::{fmt::Display, fs, path::Path, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

use crate::error::{GolemError, Result};

/// Full runtime configuration schema for a Golem.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct GolemConfig {
    /// Core identity and deployment settings.
    pub golem: GolemSection,
    /// Heartbeat timing and escalation thresholds.
    pub heartbeat: HeartbeatConfig,
    /// Inference provider routing and budget.
    pub inference: InferenceConfig,
    /// Safety policy and spending limits.
    pub safety: SafetyConfig,
    /// Custody mode and delegation bounds.
    pub custody: CustodyConfig,
    /// Styx connectivity and budgets.
    pub styx: StyxConfig,
    /// Successor creation and inheritance controls.
    pub succession: SuccessionConfig,
    /// Daimon affect engine settings.
    pub daimon: DaimonConfig,
    /// Dream scheduling and phase budgets.
    pub dreams: DreamsConfig,
    /// Prediction engine configuration.
    pub oracle: OracleConfig,
    /// Mortality subsystem configuration.
    pub mortality: MortalityConfig,
    /// Compute-tier selection.
    pub compute: ComputeConfig,
    /// mirage-rs sidecar connectivity settings.
    pub mirage: MirageSection,
}

impl GolemConfig {
    /// Loads configuration from a TOML file and applies environment overrides.
    pub fn from_file(path: &Path) -> Result<Self> {
        let source = fs::read_to_string(path)?;
        Self::from_str(&source)
    }

    /// Loads configuration from a TOML string and applies environment overrides.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self> {
        <Self as FromStr>::from_str(s)
    }

    /// Applies supported `GOLEM_*` and `BARDO_*` environment overrides.
    pub fn with_env_overrides(self) -> Result<Self> {
        self.with_lookup(|key| std::env::var(key).ok())
    }

    #[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
    fn with_lookup<F>(mut self, mut lookup: F) -> Result<Self>
    where
        F: FnMut(&str) -> Option<String>,
    {
        if let Some(value) = lookup("GOLEM_NAME") {
            self.golem.name = value;
        }
        if let Some(value) =
            parse_lookup::<StrategyCategory, _>(&mut lookup, "GOLEM_STRATEGY_CATEGORY")?
        {
            self.golem.strategy_category = value;
        }
        if let Some(value) = parse_lookup::<Network, _>(&mut lookup, "GOLEM_NETWORK")? {
            self.golem.network = value;
        }
        if let Some(value) = parse_lookup::<DeploymentMode, _>(&mut lookup, "GOLEM_MODE")? {
            self.golem.mode = value;
            self.compute.mode = value;
        }
        if let Some(value) = lookup("GOLEM_FUNDING") {
            self.golem.funding = value;
        }
        if let Some(value) = parse_lookup::<CustodyMode, _>(&mut lookup, "GOLEM_CUSTODY_MODE")? {
            self.golem.custody_mode = value;
            self.custody.mode = value;
        }
        if let Some(value) =
            parse_lookup::<TransferRestriction, _>(&mut lookup, "GOLEM_TRANSFER_RESTRICTION")?
        {
            self.golem.transfer_restriction = value;
        }
        if let Some(value) = parse_lookup::<u64, _>(&mut lookup, "GOLEM_TICK_INTERVAL")? {
            self.heartbeat.base_interval_seconds = value;
        }
        if let Some(value) = parse_lookup::<f64, _>(&mut lookup, "GOLEM_DELIBERATION_THRESHOLD")? {
            self.heartbeat.base_deliberation_threshold = value;
        }
        if let Some(value) = parse_lookup::<f64, _>(&mut lookup, "GOLEM_MAX_DAILY_COST")? {
            self.heartbeat.max_daily_cost_usd = value;
        }
        if let Some(value) =
            parse_lookup::<InferencePayment, _>(&mut lookup, "GOLEM_INFERENCE_PAYMENT")?
        {
            self.inference.payment = value;
        }
        if let Some(value) = parse_lookup::<f64, _>(&mut lookup, "GOLEM_INFERENCE_DAILY_BUDGET")? {
            self.inference.daily_budget_usd = value;
        }
        if let Some(value) = parse_lookup::<u64, _>(&mut lookup, "GOLEM_SPEND_LIMIT_TX")? {
            self.safety.spending_limits.per_transaction = value;
        }
        if let Some(value) = parse_lookup::<u64, _>(&mut lookup, "GOLEM_SPEND_LIMIT_DAILY")? {
            self.safety.spending_limits.per_day = value;
        }
        if let Some(value) = parse_bool_lookup(&mut lookup, "BARDO_STYX_ENABLED")? {
            self.styx.enabled = value;
        }
        if let Some(value) = lookup("BARDO_STYX_HOST") {
            self.styx.host = value;
        }
        if let Some(value) = parse_bool_lookup(&mut lookup, "BARDO_CLADE_ENABLED")? {
            self.styx.clade.enabled = value;
        }
        if let Some(value) = parse_lookup::<f64, _>(&mut lookup, "BARDO_STYX_DAILY_BUDGET")? {
            self.styx.budget.daily_budget = value;
        }
        if let Some(value) = parse_lookup::<f64, _>(&mut lookup, "BARDO_STYX_MONTHLY_BUDGET")? {
            self.styx.budget.monthly_budget = value;
        }
        if let Some(value) = parse_bool_lookup(&mut lookup, "GOLEM_SUCCESSION_AUTO")? {
            self.succession.auto = value;
        }
        if let Some(value) = parse_lookup::<f64, _>(&mut lookup, "GOLEM_SUCCESSION_BUDGET")? {
            self.succession.budget_usdc = value;
        }
        if let Some(value) = parse_bool_lookup(&mut lookup, "GOLEM_ORACLE_ENABLED")? {
            self.oracle.enabled = value;
        }
        if let Some(value) = parse_bool_lookup(&mut lookup, "GOLEM_DAIMON_ENABLED")? {
            self.daimon.enabled = value;
        }
        if let Some(value) =
            parse_lookup::<AppraisalModel, _>(&mut lookup, "GOLEM_APPRAISAL_MODEL")?
        {
            self.daimon.appraisal_model = value;
        }
        if let Some(value) = parse_bool_lookup(&mut lookup, "GOLEM_DREAMS_ENABLED")? {
            self.dreams.enabled = value;
        }
        if let Some(value) = parse_lookup::<DreamSchedule, _>(&mut lookup, "GOLEM_DREAM_SCHEDULE")?
        {
            self.dreams.schedule = value;
        }
        if let Some(value) = lookup("GOLEM_DREAM_INFERENCE_PROVIDER") {
            self.dreams.dream_inference_provider = Some(value);
        }
        if let Some(value) = parse_bool_lookup(&mut lookup, "BARDO_IMMORTAL")? {
            self.mortality.immortal = value;
        }
        if let Some(value) = parse_bool_lookup(&mut lookup, "BARDO_MORTALITY_ENABLED")? {
            self.mortality.economic.enabled = value;
        }
        if let Some(value) = parse_lookup::<u64, _>(&mut lookup, "BARDO_STOCHASTIC_SEED")? {
            self.mortality.stochastic.seed = Some(value);
        }
        if let Some(value) = parse_lookup::<ComputeTier, _>(&mut lookup, "GOLEM_COMPUTE_TIER")? {
            self.compute.tier = value;
        }
        if let Some(value) = lookup("BARDO_MIRAGE_URL") {
            self.mirage.url = Some(value);
        }
        if let Some(value) = lookup("BARDO_MIRAGE_HOST") {
            self.mirage.host = value;
        }
        if let Some(value) = parse_lookup::<u16, _>(&mut lookup, "BARDO_MIRAGE_PORT")? {
            self.mirage.port = value;
        }
        if let Some(value) = parse_lookup::<u64, _>(&mut lookup, "BARDO_MIRAGE_TIMEOUT_MS")? {
            self.mirage.timeout_ms = value;
        }
        if let Some(value) = parse_lookup::<u32, _>(&mut lookup, "BARDO_MIRAGE_RETRY_ATTEMPTS")? {
            self.mirage.retry_attempts = value;
        }
        if let Some(value) = parse_lookup::<u64, _>(&mut lookup, "BARDO_MIRAGE_RETRY_BACKOFF_MS")? {
            self.mirage.retry_backoff_ms = value;
        }
        Ok(self)
    }
}

fn parse_lookup<T, F>(lookup: &mut F, key: &str) -> Result<Option<T>>
where
    T: FromStr,
    T::Err: Display,
    F: FnMut(&str) -> Option<String>,
{
    lookup(key).map_or_else(
        || Ok(None),
        |value| {
            value
                .parse::<T>()
                .map(Some)
                .map_err(|error| GolemError::Config(format!("invalid {key}: {error}")))
        },
    )
}

fn parse_bool_lookup<F>(lookup: &mut F, key: &str) -> Result<Option<bool>>
where
    F: FnMut(&str) -> Option<String>,
{
    lookup(key).map_or_else(
        || Ok(None),
        |value| {
            parse_bool(&value)
                .map(Some)
                .map_err(|error| GolemError::Config(format!("invalid {key}: {error}")))
        },
    )
}

impl FromStr for GolemConfig {
    type Err = GolemError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        toml::from_str::<Self>(s)?.with_env_overrides()
    }
}

fn parse_bool(value: &str) -> std::result::Result<bool, &'static str> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err("expected boolean"),
    }
}

fn default_golem_name() -> String {
    let id = Uuid::new_v4().simple().to_string();
    format!("golem-{}", &id[..6])
}

/// Core identity block.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GolemSection {
    /// Human-readable golem name.
    pub name: String,
    /// High-level strategy category.
    pub strategy_category: StrategyCategory,
    /// Target execution network.
    pub network: Network,
    /// Runtime deployment mode.
    pub mode: DeploymentMode,
    /// Funding amount or funding descriptor.
    pub funding: String,
    /// Chosen custody mode.
    pub custody_mode: CustodyMode,
    /// Transfer restriction policy.
    pub transfer_restriction: TransferRestriction,
    /// Configuration schema version.
    pub schema_version: u32,
}

impl Default for GolemSection {
    fn default() -> Self {
        Self {
            name: default_golem_name(),
            strategy_category: StrategyCategory::Custom,
            network: Network::BaseSepolia,
            mode: DeploymentMode::Hosted,
            funding: "50.00".to_owned(),
            custody_mode: CustodyMode::Delegation,
            transfer_restriction: TransferRestriction::Strict,
            schema_version: 1,
        }
    }
}

/// High-level strategy archetype for the golem.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum StrategyCategory {
    /// Dollar-cost-averaging strategy.
    Dca,
    /// Yield-oriented strategy.
    Yield,
    /// Liquidity-provision strategy.
    Lp,
    /// Momentum-driven strategy.
    Momentum,
    /// Strategy spanning multiple protocols.
    MultiProtocol,
    /// Custom operator-defined strategy.
    Custom,
}

impl FromStr for StrategyCategory {
    type Err = &'static str;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "dca" => Ok(Self::Dca),
            "yield" => Ok(Self::Yield),
            "lp" => Ok(Self::Lp),
            "momentum" => Ok(Self::Momentum),
            "multi-protocol" | "multi_protocol" | "multiprotocol" => Ok(Self::MultiProtocol),
            "custom" => Ok(Self::Custom),
            _ => Err("expected one of: dca, yield, lp, momentum, multi-protocol, custom"),
        }
    }
}

/// Network that the golem targets for execution.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Network {
    /// Base mainnet.
    Base,
    /// Base Sepolia testnet.
    BaseSepolia,
    /// Local Anvil test chain.
    Anvil,
}

impl FromStr for Network {
    type Err = &'static str;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "base" => Ok(Self::Base),
            "base-sepolia" | "base_sepolia" | "basesepolia" => Ok(Self::BaseSepolia),
            "anvil" => Ok(Self::Anvil),
            _ => Err("expected one of: base, base-sepolia, anvil"),
        }
    }
}

/// Deployment environment selection.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum DeploymentMode {
    /// Hosted runtime environment.
    Hosted,
    /// Self-hosted runtime environment.
    SelfHosted,
}

impl FromStr for DeploymentMode {
    type Err = &'static str;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "hosted" => Ok(Self::Hosted),
            "self-hosted" | "self_hosted" | "selfhosted" => Ok(Self::SelfHosted),
            _ => Err("expected one of: hosted, self-hosted"),
        }
    }
}

/// Custody strategy for asset control.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CustodyMode {
    #[serde(rename = "delegation")]
    /// Delegated custody with bounded permissions.
    Delegation,
    #[serde(rename = "embedded")]
    /// Embedded custody managed by the runtime.
    Embedded,
    #[serde(rename = "localkey")]
    /// Local key material managed directly by the operator.
    LocalKey,
}

impl FromStr for CustodyMode {
    type Err = &'static str;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "delegation" => Ok(Self::Delegation),
            "embedded" => Ok(Self::Embedded),
            "localkey" | "local-key" | "local_key" => Ok(Self::LocalKey),
            _ => Err("expected one of: delegation, embedded, localkey"),
        }
    }
}

/// Transfer-scope restrictions applied to outbound actions.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TransferRestriction {
    /// Only explicitly approved transfers are allowed.
    Strict,
    /// Transfers are limited to the local clade.
    Clade,
    /// Transfers are limited to the active network.
    Network,
    /// Transfers are unrestricted by scope.
    Unrestricted,
}

impl FromStr for TransferRestriction {
    type Err = &'static str;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "strict" => Ok(Self::Strict),
            "clade" => Ok(Self::Clade),
            "network" => Ok(Self::Network),
            "unrestricted" => Ok(Self::Unrestricted),
            _ => Err("expected one of: strict, clade, network, unrestricted"),
        }
    }
}

/// Heartbeat timing and threshold configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HeartbeatConfig {
    /// Base interval between heartbeat ticks, in seconds.
    pub base_interval_seconds: u64,
    /// Baseline deliberation threshold.
    pub base_deliberation_threshold: f64,
    /// Maximum daily runtime cost in USD.
    pub max_daily_cost_usd: f64,
    /// Fraction of budget at which a warning is raised.
    pub cost_warning_threshold: f64,
    /// Fraction of budget at which soft-capping begins.
    pub cost_soft_cap_threshold: f64,
    /// Number of writes to batch together.
    pub write_batch_size: u32,
    /// Maximum time before a write batch is flushed, in milliseconds.
    pub write_batch_flush_interval_ms: u64,
    /// Regime-specific heartbeat interval multipliers.
    pub regime_multipliers: RegimeMultipliers,
    /// Thresholds that trigger additional probes.
    pub probe_thresholds: ProbeThresholds,
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self {
            base_interval_seconds: 15,
            base_deliberation_threshold: 0.3,
            max_daily_cost_usd: 10.0,
            cost_warning_threshold: 0.7,
            cost_soft_cap_threshold: 0.9,
            write_batch_size: 25,
            write_batch_flush_interval_ms: 30_000,
            regime_multipliers: RegimeMultipliers::default(),
            probe_thresholds: ProbeThresholds::default(),
        }
    }
}

/// Market regime classification tag.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[repr(u8)]
#[serde(rename_all = "snake_case")]
pub enum RegimeTag {
    /// Trending-up market condition.
    TrendingUp = 0,
    /// Trending-down market condition.
    TrendingDown = 1,
    /// Volatile market condition.
    Volatile = 2,
    /// Range-bound market condition.
    RangeBound = 3,
    /// Unknown or unclassified market condition.
    Unknown = 4,
}

impl RegimeTag {
    /// Converts a raw u8 to a regime tag.
    #[must_use]
    pub const fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::TrendingUp,
            1 => Self::TrendingDown,
            2 => Self::Volatile,
            3 => Self::RangeBound,
            _ => Self::Unknown,
        }
    }

    /// Returns all regime tag variants.
    #[must_use]
    pub const fn all() -> [Self; 5] {
        [
            Self::TrendingUp,
            Self::TrendingDown,
            Self::Volatile,
            Self::RangeBound,
            Self::Unknown,
        ]
    }
}

impl FromStr for RegimeTag {
    type Err = &'static str;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "trending_up" | "trending-up" | "trendingup" => Ok(Self::TrendingUp),
            "trending_down" | "trending-down" | "trendingdown" => Ok(Self::TrendingDown),
            "volatile" => Ok(Self::Volatile),
            "range_bound" | "range-bound" | "rangebound" => Ok(Self::RangeBound),
            "unknown" => Ok(Self::Unknown),
            _ => Err("expected one of: trending_up, trending_down, volatile, range_bound, unknown"),
        }
    }
}

/// Multipliers applied to heartbeat cadence per detected regime.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RegimeMultipliers {
    /// Multiplier for trending-up markets.
    pub trending_up: f64,
    /// Multiplier for trending-down markets.
    pub trending_down: f64,
    /// Multiplier for volatile markets.
    pub volatile: f64,
    /// Multiplier for range-bound markets.
    pub range_bound: f64,
    /// Multiplier for unknown regimes.
    pub unknown: f64,
}

impl RegimeMultipliers {
    /// Returns the multiplier for a given regime tag.
    #[must_use]
    pub fn for_regime(&self, tag: RegimeTag) -> f64 {
        match tag {
            RegimeTag::TrendingUp => self.trending_up,
            RegimeTag::TrendingDown => self.trending_down,
            RegimeTag::Volatile => self.volatile,
            RegimeTag::RangeBound => self.range_bound,
            RegimeTag::Unknown => self.unknown,
        }
    }
}

impl Default for RegimeMultipliers {
    fn default() -> Self {
        Self {
            trending_up: 1.0,
            trending_down: 0.5,
            volatile: 0.3,
            range_bound: 2.0,
            unknown: 0.8,
        }
    }
}

/// Probe-trigger thresholds used by heartbeat sensing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProbeThresholds {
    /// Lower price-delta threshold in basis points.
    pub price_delta_low_bps: u32,
    /// Upper price-delta threshold in basis points.
    pub price_delta_high_bps: u32,
    /// Lower health-factor threshold.
    pub health_factor_low: f64,
    /// Upper health-factor threshold.
    pub health_factor_high: f64,
    /// Lower world-model drift threshold.
    pub world_model_drift_low: f64,
    /// Upper world-model drift threshold.
    pub world_model_drift_high: f64,
}

impl Default for ProbeThresholds {
    fn default() -> Self {
        Self {
            price_delta_low_bps: 50,
            price_delta_high_bps: 200,
            health_factor_low: 1.5,
            health_factor_high: 1.2,
            world_model_drift_low: 0.10,
            world_model_drift_high: 0.25,
        }
    }
}

/// Inference provider routing configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct InferenceConfig {
    /// Payment mode used for inference requests.
    pub payment: InferencePayment,
    /// Maximum daily inference budget in USD.
    pub daily_budget_usd: f64,
    /// Ordered list of inference providers.
    pub providers: Vec<InferenceProvider>,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            payment: InferencePayment::GolemWallet,
            daily_budget_usd: 5.0,
            providers: vec![InferenceProvider::default()],
        }
    }
}

/// Payment strategy for inference requests.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InferencePayment {
    /// Bill inference to the golem wallet.
    GolemWallet,
    /// Use prepaid inference credits.
    Prepaid,
    /// Use DIEM-style delegated payment.
    Diem,
    /// Combine multiple payment sources.
    Composite,
}

impl FromStr for InferencePayment {
    type Err = &'static str;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "golem_wallet" | "golem-wallet" | "golemwallet" => Ok(Self::GolemWallet),
            "prepaid" => Ok(Self::Prepaid),
            "diem" => Ok(Self::Diem),
            "composite" => Ok(Self::Composite),
            _ => Err("expected one of: golem_wallet, prepaid, diem, composite"),
        }
    }
}

/// One concrete inference provider entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct InferenceProvider {
    #[serde(rename = "type")]
    /// Provider implementation type.
    pub provider_type: ProviderType,
    /// Whether this provider routes through DIEM.
    pub diem: bool,
}

impl Default for InferenceProvider {
    fn default() -> Self {
        Self {
            provider_type: ProviderType::Bardo,
            diem: false,
        }
    }
}

/// Supported inference provider backends.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProviderType {
    /// Bardo-managed provider.
    Bardo,
    /// Venice provider.
    Venice,
    /// Bankr provider.
    Bankr,
    /// Anthropic provider.
    Anthropic,
    #[serde(rename = "openai")]
    /// `OpenAI` provider.
    OpenAi,
    /// Google provider.
    Google,
    /// `DeepSeek` provider.
    Deepseek,
    /// Local self-hosted provider.
    Local,
}

impl FromStr for ProviderType {
    type Err = &'static str;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "bardo" => Ok(Self::Bardo),
            "venice" => Ok(Self::Venice),
            "bankr" => Ok(Self::Bankr),
            "anthropic" => Ok(Self::Anthropic),
            "openai" | "open_ai" | "open-ai" => Ok(Self::OpenAi),
            "google" => Ok(Self::Google),
            "deepseek" => Ok(Self::Deepseek),
            "local" => Ok(Self::Local),
            _ => Err(
                "expected one of: bardo, venice, bankr, anthropic, openai, google, deepseek, local",
            ),
        }
    }
}

/// Safety policy configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SafetyConfig {
    /// Allowlisted asset identifiers.
    pub approved_assets: Vec<String>,
    /// Allowlisted protocol identifiers.
    pub approved_protocols: Vec<String>,
    /// Maximum number of assets the golem may hold.
    pub max_asset_count: u32,
    /// Maximum position size in basis points.
    pub max_position_bps: u32,
    /// Maximum concentration per asset or venue in basis points.
    pub max_concentration_bps: u32,
    /// Minimum collateral ratio in basis points.
    pub min_collateral_ratio_bps: u32,
    /// Maximum drawdown in basis points.
    pub max_drawdown_bps: u32,
    /// Rolling drawdown evaluation window.
    pub drawdown_window: u64,
    /// Minimum seconds between rebalances.
    pub min_rebalance_interval: u64,
    /// Maximum number of rebalances permitted per day.
    pub max_rebalances_per_day: u32,
    /// Whether arbitrary calldata is permitted.
    pub allow_arbitrary_calldata: bool,
    /// Address or identifier of the sanctions oracle.
    pub sanction_oracle: String,
    /// Absolute spending limits.
    pub spending_limits: SpendingLimits,
}

impl Default for SafetyConfig {
    fn default() -> Self {
        Self {
            approved_assets: Vec::new(),
            approved_protocols: Vec::new(),
            max_asset_count: 10,
            max_position_bps: 2500,
            max_concentration_bps: 3000,
            min_collateral_ratio_bps: 12_500,
            max_drawdown_bps: 1300,
            drawdown_window: 86_400,
            min_rebalance_interval: 3600,
            max_rebalances_per_day: 24,
            allow_arbitrary_calldata: false,
            sanction_oracle: "0x40C57923924B5c5c5455c48D93317139ADDaC8fb".to_owned(),
            spending_limits: SpendingLimits::default(),
        }
    }
}

/// Hard spending caps enforced by the safety layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SpendingLimits {
    /// Per-transaction spending cap.
    pub per_transaction: u64,
    /// Per-session spending cap.
    pub per_session: u64,
    /// Per-day spending cap.
    pub per_day: u64,
}

impl Default for SpendingLimits {
    fn default() -> Self {
        Self {
            per_transaction: 10_000,
            per_session: 50_000,
            per_day: 100_000,
        }
    }
}

/// Custody mode configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CustodyConfig {
    /// Active custody mode.
    pub mode: CustodyMode,
    /// Optional delegation bounds when delegation is enabled.
    pub delegation_bounds: Option<DelegationBounds>,
}

impl Default for CustodyConfig {
    fn default() -> Self {
        Self {
            mode: CustodyMode::Delegation,
            delegation_bounds: None,
        }
    }
}

/// Limits applied to delegated custody.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DelegationBounds {
    /// Maximum daily delegated spend in USD.
    pub max_daily_spend_usd: f64,
    /// Maximum number of delegated calls.
    pub max_total_calls: u64,
    /// Optional delegation expiry timestamp.
    pub expires_at: Option<u64>,
    /// Allowlisted target addresses or identifiers.
    pub allowed_targets: Vec<String>,
}

impl Default for DelegationBounds {
    fn default() -> Self {
        Self {
            max_daily_spend_usd: 1000.0,
            max_total_calls: 10_000,
            expires_at: None,
            allowed_targets: Vec::new(),
        }
    }
}

/// Styx relay and knowledge-sharing configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StyxConfig {
    /// Whether Styx integration is enabled.
    pub enabled: bool,
    /// Styx host or endpoint.
    pub host: String,
    /// Vault archival settings.
    pub vault: StyxVaultConfig,
    /// Clade sync settings.
    pub clade: StyxCladeConfig,
    /// Lethe publish/read settings.
    pub lethe: StyxLetheConfig,
    /// Pheromone exchange settings.
    pub pheromone: StyxPheromoneConfig,
    /// Marketplace discovery settings.
    pub marketplace: StyxMarketplaceConfig,
    /// Styx spending budget settings.
    pub budget: StyxBudgetConfig,
}

impl Default for StyxConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            host: "styx.bardo.run".to_owned(),
            vault: StyxVaultConfig::default(),
            clade: StyxCladeConfig::default(),
            lethe: StyxLetheConfig::default(),
            pheromone: StyxPheromoneConfig::default(),
            marketplace: StyxMarketplaceConfig::default(),
            budget: StyxBudgetConfig::default(),
        }
    }
}

/// Styx vault archival configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StyxVaultConfig {
    /// Whether vault backups are enabled.
    pub enabled: bool,
    /// Time-to-live applied to stored artifacts.
    pub ttl: String,
    /// Backup interval in ticks.
    pub backup_interval_ticks: u64,
}

impl Default for StyxVaultConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ttl: "90d".to_owned(),
            backup_interval_ticks: 200,
        }
    }
}

/// Clade-sharing configuration for Styx.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StyxCladeConfig {
    /// Whether clade sync is enabled.
    pub enabled: bool,
    /// Interval between sync passes, in ticks.
    pub sync_interval_ticks: u64,
    /// Whether warnings are propagated immediately.
    pub immediate_warnings: bool,
    /// Whether bloodstains are propagated immediately.
    pub immediate_bloodstains: bool,
    /// Peer-to-peer transport settings.
    pub p2p: StyxP2pConfig,
}

impl Default for StyxCladeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sync_interval_ticks: 50,
            immediate_warnings: true,
            immediate_bloodstains: true,
            p2p: StyxP2pConfig::default(),
        }
    }
}

/// Peer-to-peer transport settings for Styx clade sync.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct StyxP2pConfig {
    /// Whether peer-to-peer sync is enabled.
    pub enabled: bool,
    /// Optional listen address for inbound peers.
    pub listen_address: Option<String>,
    /// Seed peer addresses.
    pub peers: Vec<String>,
}

/// Lethe memory-publishing configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StyxLetheConfig {
    /// Whether Lethe reads are enabled.
    pub read_enabled: bool,
    /// Whether Lethe publishes are enabled.
    pub publish_enabled: bool,
    /// Domains eligible for Lethe exchange.
    pub domains: Vec<String>,
}

impl Default for StyxLetheConfig {
    fn default() -> Self {
        Self {
            read_enabled: true,
            publish_enabled: true,
            domains: vec![
                "dex-lp".to_owned(),
                "lending".to_owned(),
                "gas".to_owned(),
                "regime".to_owned(),
            ],
        }
    }
}

/// Pheromone exchange configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StyxPheromoneConfig {
    /// Whether pheromone support is enabled.
    pub enabled: bool,
    /// Whether deposits are allowed.
    pub deposit_enabled: bool,
    /// Whether sensing is allowed.
    pub sense_enabled: bool,
}

impl Default for StyxPheromoneConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            deposit_enabled: true,
            sense_enabled: true,
        }
    }
}

/// Styx marketplace discovery and selling configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StyxMarketplaceConfig {
    /// Whether browsing the marketplace is enabled.
    pub browse_enabled: bool,
    /// Whether automatic discovery is enabled.
    pub auto_discover: bool,
    /// Maximum amount available for automatic spending.
    pub max_auto_spend: f64,
    /// Minimum seller reputation required for automatic discovery.
    pub min_seller_reputation: u32,
    /// Whether selling is enabled.
    pub sell_enabled: bool,
}

impl Default for StyxMarketplaceConfig {
    fn default() -> Self {
        Self {
            browse_enabled: true,
            auto_discover: true,
            max_auto_spend: 1.0,
            min_seller_reputation: 60,
            sell_enabled: false,
        }
    }
}

/// Budget controls for Styx interactions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StyxBudgetConfig {
    /// Maximum spend per tick.
    pub max_per_tick: f64,
    /// Maximum daily budget.
    pub daily_budget: f64,
    /// Maximum monthly budget.
    pub monthly_budget: f64,
}

impl Default for StyxBudgetConfig {
    fn default() -> Self {
        Self {
            max_per_tick: 0.01,
            daily_budget: 0.50,
            monthly_budget: 10.00,
        }
    }
}

/// Successor-creation settings.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SuccessionConfig {
    /// Whether succession support is enabled.
    pub enabled: bool,
    /// Whether successor spawning may happen automatically.
    pub auto: bool,
    /// Budget available for successor creation, in USDC.
    pub budget_usdc: f64,
    /// Maximum allowed strategy drift from the parent.
    pub strategy_drift_allowed: f64,
    /// Whether the successor inherits grimoire state.
    pub inherit_grimoire: bool,
    /// Minimum confidence required for inheritance.
    pub inheritance_confidence: f64,
    /// Minimum playbook divergence required for spawning.
    pub min_playbook_divergence: f64,
    /// Whether inheritance may source from the clade.
    pub inherit_from_clade: bool,
    /// Whether novelty ranking influences successor choice.
    pub use_novelty_ranking: bool,
}

impl Default for SuccessionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto: false,
            budget_usdc: 50.0,
            strategy_drift_allowed: 0.3,
            inherit_grimoire: true,
            inheritance_confidence: 0.4,
            min_playbook_divergence: 0.15,
            inherit_from_clade: true,
            use_novelty_ranking: true,
        }
    }
}

/// Affect-engine configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DaimonConfig {
    /// Whether the daimon subsystem is enabled.
    pub enabled: bool,
    /// Appraisal model to use.
    pub appraisal_model: AppraisalModel,
    /// Mood decay applied per update.
    pub mood_decay_rate: f64,
    /// Whether appraisal incorporates mortality signals.
    pub mortality_aware_affect: bool,
    /// Duration of grief responses, in ticks.
    pub grief_duration_ticks: u32,
    /// Whether emotional context is recorded.
    pub record_emotional_context: bool,
}

impl Default for DaimonConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            appraisal_model: AppraisalModel::ChainOfEmotion,
            mood_decay_rate: 0.95,
            mortality_aware_affect: true,
            grief_duration_ticks: 100,
            record_emotional_context: true,
        }
    }
}

/// Appraisal-model selection for the daimon.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AppraisalModel {
    /// Chain-of-emotion appraisal model.
    ChainOfEmotion,
    /// Rule-based appraisal model.
    RuleBased,
    /// Disable appraisal updates.
    Disabled,
}

impl FromStr for AppraisalModel {
    type Err = &'static str;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "chain_of_emotion" | "chain-of-emotion" => Ok(Self::ChainOfEmotion),
            "rule_based" | "rule-based" | "rulebased" => Ok(Self::RuleBased),
            "disabled" => Ok(Self::Disabled),
            _ => Err("expected one of: chain_of_emotion, rule_based, disabled"),
        }
    }
}

/// Dream-cycle configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DreamsConfig {
    /// Whether dream processing is enabled.
    pub enabled: bool,
    /// Scheduling mode for dream cycles.
    pub schedule: DreamSchedule,
    /// Autonomy score required to self-trigger dreaming.
    pub autonomous_threshold: f64,
    /// Budget fraction reserved for dream work.
    pub budget_fraction: f64,
    /// Vitality threshold below which dreaming stops.
    pub terminal_cutoff: f64,
    /// Maximum staged revisions per dream cycle.
    pub max_staged_revisions: u32,
    /// Optional provider override for dream inference.
    pub dream_inference_provider: Option<String>,
    /// Web-search budget per dream cycle, in USDC.
    pub web_search_budget_per_cycle_usdc: f64,
    #[serde(default, deserialize_with = "deserialize_dream_windows")]
    /// Allowed dream windows.
    pub windows: Vec<DreamWindow>,
    /// Relative phase scaling weights.
    pub phase_scaling: DreamPhaseScaling,
}

impl Default for DreamsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            schedule: DreamSchedule::Hybrid,
            autonomous_threshold: 0.8,
            budget_fraction: 0.08,
            terminal_cutoff: 0.15,
            max_staged_revisions: 10,
            dream_inference_provider: None,
            web_search_budget_per_cycle_usdc: 0.05,
            windows: vec![DreamWindow::default()],
            phase_scaling: DreamPhaseScaling::default(),
        }
    }
}

/// Scheduling strategy for dream cycles.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DreamSchedule {
    /// Dreams run only on operator request.
    Operator,
    /// Dreams run autonomously.
    Autonomous,
    /// Dreams may run either autonomously or by operator trigger.
    Hybrid,
}

impl FromStr for DreamSchedule {
    type Err = &'static str;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "operator" => Ok(Self::Operator),
            "autonomous" => Ok(Self::Autonomous),
            "hybrid" => Ok(Self::Hybrid),
            _ => Err("expected one of: operator, autonomous, hybrid"),
        }
    }
}

/// One scheduled dream window.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DreamWindow {
    /// Window start time.
    pub start: String,
    /// Window end time.
    pub end: String,
    /// Timezone used to interpret the window.
    pub timezone: String,
}

impl Default for DreamWindow {
    fn default() -> Self {
        Self {
            start: "02:00".to_owned(),
            end: "06:00".to_owned(),
            timezone: "UTC".to_owned(),
        }
    }
}

/// Relative weights for dream sub-phases.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DreamPhaseScaling {
    /// Scaling weight for NREM-style processing.
    pub nrem: f64,
    /// Scaling weight for REM-style processing.
    pub rem: f64,
    /// Scaling weight for integration passes.
    pub integration: f64,
}

impl Default for DreamPhaseScaling {
    fn default() -> Self {
        Self {
            nrem: 0.45,
            rem: 0.35,
            integration: 0.20,
        }
    }
}

fn deserialize_dream_windows<'de, D>(
    deserializer: D,
) -> std::result::Result<Vec<DreamWindow>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum DreamWindowsValue {
        Direct(Vec<DreamWindow>),
        Wrapped { windows: Vec<DreamWindow> },
    }

    Ok(
        match Option::<DreamWindowsValue>::deserialize(deserializer)? {
            Some(DreamWindowsValue::Direct(windows) | DreamWindowsValue::Wrapped { windows }) => {
                windows
            }
            None => Vec::new(),
        },
    )
}

/// Prediction-engine configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OracleConfig {
    /// Whether the oracle subsystem is enabled.
    pub enabled: bool,
    /// Size of the residual buffer.
    pub residual_buffer_size: u32,
    /// Target predictive coverage ratio.
    pub target_coverage: f64,
    /// Minimum correction samples before adaptation.
    pub min_correction_samples: u32,
    /// Novelty threshold for flagging predictions.
    pub novelty_threshold: f64,
    /// Forgetting rate applied during adaptation.
    pub forgetting_rate: f64,
    /// Compaction window for oracle state.
    pub compaction_window: u64,
    /// Attention-management settings.
    pub attention: OracleAttentionConfig,
    /// Decision-gate settings.
    pub gate: OracleGateConfig,
    /// Calibration settings.
    pub calibration: OracleCalibrationConfig,
}

impl Default for OracleConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            residual_buffer_size: 256,
            target_coverage: 0.85,
            min_correction_samples: 10,
            novelty_threshold: 2.0,
            forgetting_rate: 0.005,
            compaction_window: 604_800,
            attention: OracleAttentionConfig::default(),
            gate: OracleGateConfig::default(),
            calibration: OracleCalibrationConfig::default(),
        }
    }
}

/// Attention-allocation settings for the oracle.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OracleAttentionConfig {
    /// Maximum active items.
    pub active_max: u32,
    /// Maximum watched items.
    pub watched_max: u32,
    /// Maximum scanned items.
    pub scanned_max: u32,
    /// Evaluation frequency for watched items.
    pub watched_eval_frequency: u32,
    /// Evaluation frequency for scanned items.
    pub scanned_eval_frequency: u32,
    /// Threshold required for promotion.
    pub promotion_threshold: f64,
    /// Number of misses tolerated before demotion.
    pub demotion_patience: u32,
}

impl Default for OracleAttentionConfig {
    fn default() -> Self {
        Self {
            active_max: 15,
            watched_max: 60,
            scanned_max: 500,
            watched_eval_frequency: 4,
            scanned_eval_frequency: 100,
            promotion_threshold: 3.0,
            demotion_patience: 10,
        }
    }
}

/// Gate thresholds for deciding whether to act on predictions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OracleGateConfig {
    /// Minimum category confidence required to act.
    pub category_threshold: f64,
    /// Whether to compare against an inaction baseline.
    pub inaction_comparison: bool,
    /// Margin required over inaction.
    pub inaction_margin: f64,
    /// Coefficient applied to inherited priors.
    pub inheritance_coefficient: f64,
}

impl Default for OracleGateConfig {
    fn default() -> Self {
        Self {
            category_threshold: 0.60,
            inaction_comparison: true,
            inaction_margin: 0.05,
            inheritance_coefficient: 0.70,
        }
    }
}

/// Calibration schedule for the oracle.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OracleCalibrationConfig {
    /// Whether calibration is enabled.
    pub enabled: bool,
    /// Minimum samples required before refitting.
    pub min_samples: u32,
    /// Interval between refits.
    pub refit_interval: u64,
}

impl Default for OracleCalibrationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_samples: 30,
            refit_interval: 50,
        }
    }
}

/// Mortality-system configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct MortalityConfig {
    /// Whether mortality is globally disabled.
    pub immortal: bool,
    /// Economic mortality controls.
    pub economic: EconomicMortalityConfig,
    /// Epistemic mortality controls.
    pub epistemic: EpistemicMortalityConfig,
    /// Stochastic mortality controls.
    pub stochastic: StochasticMortalityConfig,
    /// Demurrage controls.
    pub demurrage: DemurrageConfig,
    /// Phage-spawning controls.
    pub phage: PhageConfig,
    /// Thanatopsis reflection controls.
    pub thanatopsis: ThanatopsisConfig,
}

/// Economic-mortality parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EconomicMortalityConfig {
    /// Whether economic mortality is enabled.
    pub enabled: bool,
    /// Absolute death-reserve floor in USDC.
    pub death_reserve_floor_usdc: f64,
    /// Proportional death-reserve fraction.
    pub death_reserve_proportional: f64,
    /// Conservation-phase threshold.
    pub conservation_threshold: f64,
    /// Terminal-phase threshold.
    pub terminal_threshold: f64,
    /// Resource partitions for steady-state budgeting.
    pub partitions: MortalityPartitionConfig,
    /// Daily caps for resource classes.
    pub daily_caps: MortalityPartitionConfig,
}

impl Default for EconomicMortalityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            death_reserve_floor_usdc: 0.30,
            death_reserve_proportional: 0.02,
            conservation_threshold: 0.30,
            terminal_threshold: 0.05,
            partitions: MortalityPartitionConfig {
                llm: 0.60,
                gas: 0.25,
                data: 0.15,
            },
            daily_caps: MortalityPartitionConfig {
                llm: 0.20,
                gas: 0.30,
                data: 0.25,
            },
        }
    }
}

/// Budget split across mortality resource classes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MortalityPartitionConfig {
    /// Language-model allocation.
    pub llm: f64,
    /// Gas allocation.
    pub gas: f64,
    /// Data allocation.
    pub data: f64,
}

impl Default for MortalityPartitionConfig {
    fn default() -> Self {
        Self {
            llm: 0.0,
            gas: 0.0,
            data: 0.0,
        }
    }
}

/// Epistemic-mortality parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EpistemicMortalityConfig {
    /// Whether epistemic mortality is enabled.
    pub enabled: bool,
    /// Number of samples in the fitness window.
    pub fitness_window: u32,
    /// Senescence threshold for prediction fitness.
    pub senescence_threshold: f64,
    /// Grace period before recovery is reconsidered.
    pub recovery_grace_period: u32,
    /// Minimum predictions required for evaluation.
    pub minimum_predictions: u32,
    /// Prediction categories tracked for epistemic fitness.
    pub tracked_prediction_types: Vec<String>,
    /// Minimum trade count required for PnL-based fitness.
    pub pnl_minimum_trades: u32,
    /// Trade window used for PnL-based fitness.
    pub pnl_fitness_window: u32,
}

impl Default for EpistemicMortalityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            fitness_window: 2000,
            senescence_threshold: 0.35,
            recovery_grace_period: 500,
            minimum_predictions: 50,
            tracked_prediction_types: vec![
                "price_direction".to_owned(),
                "volatility_regime".to_owned(),
                "fee_rate".to_owned(),
                "liquidity_depth".to_owned(),
            ],
            pnl_minimum_trades: 10,
            pnl_fitness_window: 200,
        }
    }
}

/// Stochastic-mortality parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StochasticMortalityConfig {
    /// Whether stochastic mortality is enabled.
    pub enabled: bool,
    /// Baseline stochastic hazard rate.
    pub base_hazard_rate: f64,
    /// Age-based hazard coefficient.
    pub age_hazard_coefficient: f64,
    /// Multiplier applied from epistemic decline.
    pub epistemic_hazard_multiplier: f64,
    /// Maximum allowed hazard rate.
    pub max_hazard_rate: f64,
    /// Optional deterministic random seed.
    pub seed: Option<u64>,
}

impl Default for StochasticMortalityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            base_hazard_rate: 1e-6,
            age_hazard_coefficient: 0.000_01,
            epistemic_hazard_multiplier: 2.0,
            max_hazard_rate: 0.001,
            seed: None,
        }
    }
}

/// Demurrage settings for memory and artifact decay.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DemurrageConfig {
    /// Whether demurrage is enabled.
    pub enabled: bool,
    /// Validation interval in ticks.
    pub validation_interval_ticks: u32,
    /// Decay rate applied per interval.
    pub decay_rate_per_interval: f64,
    /// Threshold below which entries are archived.
    pub archive_threshold: f64,
    /// Per-class decay configuration.
    pub decay_classes: DemurrageDecayClasses,
}

impl Default for DemurrageConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            validation_interval_ticks: 5000,
            decay_rate_per_interval: 0.05,
            archive_threshold: 0.1,
            decay_classes: DemurrageDecayClasses::default(),
        }
    }
}

/// Demurrage rates by knowledge class.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DemurrageDecayClasses {
    /// Optional structural-memory decay period.
    pub structural: Option<f64>,
    /// Decay period for regime-conditional knowledge.
    pub regime_conditional: f64,
    /// Decay period for tactical knowledge.
    pub tactical: f64,
    /// Decay period for ephemeral knowledge.
    pub ephemeral: f64,
}

impl Default for DemurrageDecayClasses {
    fn default() -> Self {
        Self {
            structural: None,
            regime_conditional: 14.0,
            tactical: 7.0,
            ephemeral: 1.0,
        }
    }
}

/// Phage-spawning configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PhageConfig {
    /// Whether phage spawning is enabled.
    pub enabled: bool,
    /// Maximum phages that may spawn per day.
    pub daily_spawn_rate: u32,
    /// Maximum phage lifetime in ticks.
    pub max_lifetime_ticks: u32,
    /// Budget allocated per phage.
    pub budget_per_phage: f64,
    /// Minimum vitality required before spawning.
    pub minimum_vitality_to_spawn: f64,
}

impl Default for PhageConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            daily_spawn_rate: 2,
            max_lifetime_ticks: 50,
            budget_per_phage: 0.05,
            minimum_vitality_to_spawn: 0.5,
        }
    }
}

/// End-of-life reflection and packaging settings.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ThanatopsisConfig {
    /// Whether thanatopsis is enabled.
    pub enabled: bool,
    /// Fraction of budget reserved for reflection.
    pub reflect_budget_fraction: f64,
    /// Whether emotional life review is included.
    pub emotional_life_review: bool,
    /// Whether unresolved questions are included.
    pub include_unresolved_questions: bool,
    /// Maximum reflection token budget.
    pub max_reflection_tokens: u32,
    /// Interval between snapshot captures, in ticks.
    pub snapshot_interval_ticks: u32,
    /// Whether to generate successor recommendations.
    pub generate_successor_recommendation: bool,
    /// Whether packaging prioritizes novelty.
    pub novelty_prioritized_packaging: bool,
}

impl Default for ThanatopsisConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            reflect_budget_fraction: 0.60,
            emotional_life_review: true,
            include_unresolved_questions: true,
            max_reflection_tokens: 8000,
            snapshot_interval_ticks: 50,
            generate_successor_recommendation: true,
            novelty_prioritized_packaging: true,
        }
    }
}

/// Compute-tier selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ComputeConfig {
    /// Deployment mode mirrored into compute selection.
    pub mode: DeploymentMode,
    /// Selected compute tier.
    pub tier: ComputeTier,
}

impl Default for ComputeConfig {
    fn default() -> Self {
        Self {
            mode: DeploymentMode::Hosted,
            tier: ComputeTier::Small,
        }
    }
}

/// Compute capacity tier.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ComputeTier {
    /// Minimal compute footprint.
    Micro,
    /// Small default compute footprint.
    Small,
    /// Medium compute footprint.
    Medium,
    /// Large compute footprint.
    Large,
}

impl FromStr for ComputeTier {
    type Err = &'static str;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "micro" => Ok(Self::Micro),
            "small" => Ok(Self::Small),
            "medium" => Ok(Self::Medium),
            "large" => Ok(Self::Large),
            _ => Err("expected one of: micro, small, medium, large"),
        }
    }
}

/// mirage-rs sidecar connectivity and retry settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MirageSection {
    /// Optional fully qualified Mirage URL.
    pub url: Option<String>,
    /// Host used when `url` is omitted.
    pub host: String,
    /// Port used when `url` is omitted.
    pub port: u16,
    /// Per-request timeout in milliseconds.
    pub timeout_ms: u64,
    /// Retry attempts on transport failure.
    pub retry_attempts: u32,
    /// Initial retry backoff in milliseconds.
    pub retry_backoff_ms: u64,
}

impl Default for MirageSection {
    fn default() -> Self {
        Self {
            url: None,
            host: "127.0.0.1".to_owned(),
            port: 8545,
            timeout_ms: 30_000,
            retry_attempts: 3,
            retry_backoff_ms: 500,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{ComputeTier, DeploymentMode, GolemConfig, RegimeTag};

    #[test]
    fn config_from_str_minimal() {
        let config = GolemConfig::from_str("").expect("parse minimal config");
        assert_eq!(config.heartbeat.base_interval_seconds, 15);
        assert_eq!(config.inference.daily_budget_usd, 5.0);
        assert_eq!(config.compute.tier, ComputeTier::Small);
        assert_eq!(config.mirage.host, "127.0.0.1");
        assert_eq!(config.mirage.port, 8545);
        assert!(!config.golem.name.is_empty());
    }

    #[test]
    fn config_env_override() {
        let env = HashMap::from([
            ("GOLEM_NAME", "oracle-7".to_owned()),
            ("GOLEM_TICK_INTERVAL", "42".to_owned()),
            ("GOLEM_MODE", "self-hosted".to_owned()),
            ("BARDO_IMMORTAL", "true".to_owned()),
            ("GOLEM_COMPUTE_TIER", "large".to_owned()),
        ]);

        let config = GolemConfig::default()
            .with_lookup(|key| env.get(key).cloned())
            .expect("apply overrides");

        assert_eq!(config.golem.name, "oracle-7");
        assert_eq!(config.heartbeat.base_interval_seconds, 42);
        assert_eq!(config.golem.mode, DeploymentMode::SelfHosted);
        assert_eq!(config.compute.mode, DeploymentMode::SelfHosted);
        assert!(config.mortality.immortal);
        assert_eq!(config.compute.tier, ComputeTier::Large);
    }

    #[test]
    fn config_parses_mirage_section() {
        let config = GolemConfig::from_str(
            r#"
                [mirage]
                url = "http://127.0.0.1:18545"
                port = 18545
                timeout_ms = 45000
                retry_attempts = 5
                retry_backoff_ms = 750
            "#,
        )
        .expect("parse mirage section");

        assert_eq!(config.mirage.url.as_deref(), Some("http://127.0.0.1:18545"));
        assert_eq!(config.mirage.port, 18_545);
        assert_eq!(config.mirage.timeout_ms, 45_000);
        assert_eq!(config.mirage.retry_attempts, 5);
        assert_eq!(config.mirage.retry_backoff_ms, 750);
    }

    #[test]
    fn config_env_override_updates_mirage_settings() {
        let env = HashMap::from([
            ("BARDO_MIRAGE_URL", "http://127.0.0.1:28545".to_owned()),
            ("BARDO_MIRAGE_PORT", "28545".to_owned()),
            ("BARDO_MIRAGE_TIMEOUT_MS", "1500".to_owned()),
            ("BARDO_MIRAGE_RETRY_ATTEMPTS", "4".to_owned()),
            ("BARDO_MIRAGE_RETRY_BACKOFF_MS", "125".to_owned()),
        ]);

        let config = GolemConfig::default()
            .with_lookup(|key| env.get(key).cloned())
            .expect("apply mirage overrides");

        assert_eq!(config.mirage.url.as_deref(), Some("http://127.0.0.1:28545"));
        assert_eq!(config.mirage.port, 28_545);
        assert_eq!(config.mirage.timeout_ms, 1_500);
        assert_eq!(config.mirage.retry_attempts, 4);
        assert_eq!(config.mirage.retry_backoff_ms, 125);
    }

    #[test]
    fn test_regime_tag_transitions() {
        assert_eq!(RegimeTag::from_u8(0), RegimeTag::TrendingUp);
        assert_eq!(RegimeTag::from_u8(1), RegimeTag::TrendingDown);
        assert_eq!(RegimeTag::from_u8(2), RegimeTag::Volatile);
        assert_eq!(RegimeTag::from_u8(3), RegimeTag::RangeBound);
        assert_eq!(RegimeTag::from_u8(4), RegimeTag::Unknown);
        assert_eq!(RegimeTag::from_u8(255), RegimeTag::Unknown);

        let config = GolemConfig::default();
        let m = &config.heartbeat.regime_multipliers;
        assert_eq!(m.for_regime(RegimeTag::TrendingUp), 1.0);
        assert_eq!(m.for_regime(RegimeTag::TrendingDown), 0.5);
        assert_eq!(m.for_regime(RegimeTag::Volatile), 0.3);
        assert_eq!(m.for_regime(RegimeTag::RangeBound), 2.0);
        assert_eq!(m.for_regime(RegimeTag::Unknown), 0.8);
    }

    #[test]
    fn test_inheritance_confidence_cap() {
        let config = GolemConfig::default();
        assert!(
            config.succession.inheritance_confidence <= 0.7,
            "inheritance_confidence must not exceed protocol cap of 0.7"
        );
        assert!(
            config.oracle.gate.inheritance_coefficient <= 0.7,
            "inheritance_coefficient must not exceed protocol cap of 0.7"
        );
    }

    #[test]
    fn test_regime_multipliers_complete() {
        let config = GolemConfig::default();
        let m = &config.heartbeat.regime_multipliers;
        for tag in RegimeTag::all() {
            assert!(
                m.for_regime(tag) > 0.0,
                "regime {tag:?} must have a positive multiplier"
            );
        }
        assert_eq!(RegimeTag::all().len(), 5);
    }

    #[test]
    fn test_probe_threshold_ordering() {
        let config = GolemConfig::default();
        let p = &config.heartbeat.probe_thresholds;
        assert!(
            p.price_delta_low_bps < p.price_delta_high_bps,
            "price_delta_low_bps ({}) must be < price_delta_high_bps ({})",
            p.price_delta_low_bps,
            p.price_delta_high_bps
        );
        assert!(
            p.health_factor_low > p.health_factor_high,
            "health_factor_low ({}) must be > health_factor_high ({}) (inverted scale)",
            p.health_factor_low,
            p.health_factor_high
        );
        assert!(
            p.world_model_drift_low < p.world_model_drift_high,
            "world_model_drift_low ({}) must be < world_model_drift_high ({})",
            p.world_model_drift_low,
            p.world_model_drift_high
        );
    }
}
