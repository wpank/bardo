use super::Intervention;

/// Build a steering message from an intervention
pub fn steering_message(intervention: &Intervention) -> String {
    format!(
        "[Monitor: {}] {}\n\nContinue from where you left off.",
        intervention.pattern,
        intervention.message
    )
}
