//! Store management operations.
pub(crate) use self::sealed::ManageOps;
pub(crate) use self::sealed::ManageResponses;
use self::sealed::SealManageOp;

/// Implementation detail to allow a single [`Store::manage`] method to handle different models.
///
/// [`Store::manage`]: super::Store::manage
pub trait ManageOp: Into<ManageOps> + SealManageOp {
    /// The return type of running a specific management operation.
    type Response: From<ManageResponses>;
}

/// Clean all actions finished prior to the given time.
pub struct CleanActions {
    age: time::OffsetDateTime,
}
impl SealManageOp for CleanActions {}
impl ManageOp for CleanActions {
    type Response = ();
}
impl From<CleanActions> for ManageOps {
    fn from(value: CleanActions) -> Self {
        ManageOps::CleanActions(value.age)
    }
}

impl CleanActions {
    /// Clean actions finished before the given time.
    pub fn since(age: time::OffsetDateTime) -> Self {
        CleanActions { age }
    }
}

/// Private module to seal as many implementation details as possible.
mod sealed {
    /// Super-trait to seal the [`ManageOp`](super::ManageOp) trait.
    pub trait SealManageOp {}

    /// Enumeration of all supported management operations.
    pub enum ManageOps {
        /// Clean all actions finished prior to the given time.
        CleanActions(time::OffsetDateTime),
    }

    /// Enumeration of responses for all supported management operations.
    pub enum ManageResponses {
        Success,
    }

    // --- Implement conversions for external types to enable transparent use ---
    impl From<ManageResponses> for () {
        fn from(value: ManageResponses) -> Self {
            match value {
                ManageResponses::Success => (),
                //_ => panic!("unexpected result type for the given management operation"),
            }
        }
    }
}
