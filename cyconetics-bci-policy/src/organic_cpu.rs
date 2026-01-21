// Example artifact: SchedulerDecision enum
/// The possible outcomes of a scheduling decision.
#[derive(Debug, Clone, PartialEq)]
pub enum SchedulerDecision {
    Authorize,
    Defer, // Retry later with a backoff strategy
    Reject, // Permanently disallow based on policy
    Escalate, // RoH ceiling has been breached
}
