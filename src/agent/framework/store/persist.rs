//! Store persistence operations.
use crate::agent::models::ActionExecution;

pub(crate) use self::sealed::PersistOps;
pub(crate) use self::sealed::PersistResponses;
use self::sealed::SealPersistOp;

/// Implementation detail to allow a single [`Store::persist`] method to handle different models.
///
/// [`Store::persist`]: super::Store::persist
pub trait PersistOp: Into<PersistOps> + SealPersistOp {
    /// The return type of running a specific query operation.
    type Response: From<PersistResponses>;
}

/// Private module to seal as many implementation details as possible.
mod sealed {
    use crate::agent::models::ActionExecution;

    /// Super-trait to seal the [`PersistOp`](super::PersistOp) trait.
    pub trait SealPersistOp {}

    /// Enumeration of all supported query operations.
    pub enum PersistOps {
        /// Create or update an [`ActionExecution`] records.
        ActionExecution(ActionExecution),
    }

    /// Enumeration of possible responses for all supported persist operations.
    pub enum PersistResponses {
        /// The persist operation does not return data but only success or failure.
        Success,
    }

    // --- Implement conversions for external types to enable transparent use ---
    impl From<PersistResponses> for () {
        fn from(value: PersistResponses) -> Self {
            match value {
                PersistResponses::Success => (),
                //_ => panic!("only PersistResponses::Success can be converted to the unit type"),
            }
        }
    }

    impl From<ActionExecution> for PersistOps {
        fn from(value: ActionExecution) -> Self {
            PersistOps::ActionExecution(value)
        }
    }
}

// --- Implement traits for external types to enable transparent use ---
impl PersistOp for ActionExecution {
    type Response = ();
}
impl SealPersistOp for ActionExecution {}
