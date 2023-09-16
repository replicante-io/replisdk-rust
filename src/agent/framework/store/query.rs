//! Store querying operations.
use crate::agent::models::ActionExecution;
use crate::agent::models::ActionExecutionList;

pub(crate) use self::sealed::QueryOps;
pub(crate) use self::sealed::QueryResponses;
use self::sealed::SealQueryOp;

/// Implementation detail to allow a single [`Store::query`] method to handle different models.
///
/// [`Store::query`]: super::Store::query
pub trait QueryOp: Into<QueryOps> + SealQueryOp {
    /// The return type of running a specific query operation.
    type Response: From<QueryResponses>;
}

/// Lookup an [`ActionExecution`] by ID.
pub struct Action {
    /// ID of the action to lookup.
    pub id: uuid::Uuid,
}
impl SealQueryOp for Action {}
impl QueryOp for Action {
    type Response = Option<ActionExecution>;
}
impl From<Action> for QueryOps {
    fn from(value: Action) -> Self {
        QueryOps::Action(value.id)
    }
}

impl Action {
    /// Query for the [`ActionExecution`] record with the given ID.
    pub fn new<U>(id: U) -> Action
    where
        U: Into<uuid::Uuid>,
    {
        let id = id.into();
        Action { id }
    }
}

/// Query the store for the next [`ActionExecution`] record to execute.
///
/// `ActionExecution`s are processed based on the time they were scheduled
/// with a preference for already running actions.
pub struct ActionNextToExecute {}
impl SealQueryOp for ActionNextToExecute {}
impl QueryOp for ActionNextToExecute {
    type Response = Option<ActionExecution>;
}
impl From<ActionNextToExecute> for QueryOps {
    fn from(_: ActionNextToExecute) -> Self {
        QueryOps::ActionNextToExecute
    }
}

/// Query the store for a list of finished [`ActionExecution`] records.
///
/// [`ActionExecution`]: crate::agent::models::ActionExecution
pub struct ActionsFinished {}
impl SealQueryOp for ActionsFinished {}
impl QueryOp for ActionsFinished {
    type Response = ActionExecutionList;
}
impl From<ActionsFinished> for QueryOps {
    fn from(_: ActionsFinished) -> Self {
        QueryOps::ActionsFinished
    }
}

/// Query the store for a list of running and queued [`ActionExecution`] records.
///
/// [`ActionExecution`]: crate::agent::models::ActionExecution
pub struct ActionsQueue {}
impl SealQueryOp for ActionsQueue {}
impl QueryOp for ActionsQueue {
    type Response = ActionExecutionList;
}
impl From<ActionsQueue> for QueryOps {
    fn from(_: ActionsQueue) -> Self {
        QueryOps::ActionsQueue
    }
}

/// Private module to seal as many implementation details as possible.
mod sealed {
    use crate::agent::models::ActionExecution;
    use crate::agent::models::ActionExecutionList;

    /// Super-trait to seal the [`QueryOp`](super::QueryOp) trait.
    pub trait SealQueryOp {}

    /// Enumeration of all supported query operations.
    pub enum QueryOps {
        /// Lookup an [`ActionExecution`] record by ID.
        Action(uuid::Uuid),

        /// Query the store for the next [`ActionExecution`] record to execute.
        ActionNextToExecute,

        /// List running and queued [`ActionExecution`] records.
        ActionsQueue,

        /// List finished [`ActionExecution`] records.
        ActionsFinished,
    }

    /// Enumeration of query responses for all supported query operations.
    pub enum QueryResponses {
        /// Result of an [`ActionExecution`] lookup query.
        Action(Option<ActionExecution>),

        /// List of [`ActionExecution`] record summaries.
        ActionsList(ActionExecutionList),
    }

    // --- Implement conversions for external types to enable transparent use ---
    impl From<QueryResponses> for ActionExecutionList {
        fn from(value: QueryResponses) -> Self {
            match value {
                QueryResponses::ActionsList(value) => value,
                _ => panic!("unexpected result type for the given query operation"),
            }
        }
    }

    impl From<QueryResponses> for Option<ActionExecution> {
        fn from(value: QueryResponses) -> Self {
            match value {
                QueryResponses::Action(value) => value,
                _ => panic!("unexpected result type for the given query operation"),
            }
        }
    }
}
