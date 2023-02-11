//! Default `Context` definition for Platform framework.
use slog::Logger;

/// Default additional context for [`IPlatform`](super::IPlatform) implementations.
///
/// When using custom contexts you can still reuse the default logic by embedding this
/// struct as a field to your custom context type.
pub struct DefaultContext {
    /// Contextual logger to be used by the operation.
    pub logger: Logger,
}
