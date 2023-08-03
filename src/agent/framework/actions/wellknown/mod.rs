//! Helpers to register well-known and ready to use action handlers.
use crate::agent::framework::actions::ActionMetadata;

mod tests;

/// Collection of actions metadata for the `agent.replicnate.io/test.*` group.
///
/// Register actions during agent initialisation with
///
/// ```ignore
/// Agent::build()
///     .register_actions(crate::agent::framework::actions::wellknown::tests())
/// ```
pub fn tests() -> impl IntoIterator<Item = ActionMetadata> {
    [
        self::tests::Fail::metadata(),
        self::tests::Loop::metadata(),
        self::tests::Success::metadata(),
    ]
}
