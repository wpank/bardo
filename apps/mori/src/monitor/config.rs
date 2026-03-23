/// Monitor configuration
#[derive(Debug, Clone)]
pub struct MonitorConfig {
    /// Cooldown between firing the same pattern (seconds)
    pub cooldown_secs: u64,
    /// Whether Tier 2 LLM monitor is enabled
    pub llm_monitor_enabled: bool,
    /// Context window size for pressure detection
    pub context_window_size: u64,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            cooldown_secs: 300,
            llm_monitor_enabled: false,
            context_window_size: 200_000,
        }
    }
}
