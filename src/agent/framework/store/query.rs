//! Store querying operations.
use self::sealed::SealQueryOp;

pub(crate) use self::sealed::QueryOps;
pub(crate) use self::sealed::QueryResponses;

/// Implementation detail to allow a single [`Store::query`] method to handle different models.
///
/// [`Store::query`]: super::Store::query
pub trait QueryOp: Into<QueryOps> + SealQueryOp {
    /// The return type of running a specific query operation.
    type Response: From<QueryResponses>;
}

/// Private module to seal as many implementation details as possible.
mod sealed {
    /// Super-trait to seal the [`QueryOp`](super::QueryOp) trait.
    pub trait SealQueryOp {}

    /// Enumeration of all supported query operations.
    pub enum QueryOps {
        // TODO
    }

    /// Enumeration of query responses for all supported query operations.
    pub enum QueryResponses {
        // TODO
    }
}
