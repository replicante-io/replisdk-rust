//! Models to describe node searches.
use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;

use super::AttributeMatcher;

/// Description of how to select a set of cluster nodes to target by control logic.
///
/// Many cluster management tasks target one or more nodes within the cluster itself.
/// Examples of this are:
///
/// - Act on cluster membership changes.
/// - Backup source selection.
///
/// Node searches enable configuration of how to select the nodes these operations should
/// target and in which ordering these should be targeted.
///
/// ## `Node`s order
///
/// A key concept is that `Node`s do not have a natural order:
/// different situations and uses call for a different orders, often user specified.
///
/// For these reasons the `Node` comparison implementation logic must make some choices.
/// While these choices are largely opinion-based it is more important for these rules
/// to be clearly set and consistently applied.
///
/// ### Attribute presence
///
/// The first choice to make is comparing attributes only set on one node:
/// value existence takes precedence over missing attributes.
///
/// - If the attribute is missing from both nodes the two are equal.
/// - If only one node has the value that node is "smaller" (ordered first).
///
/// ### Type order
///
/// When both nodes have the sorting attribute set there is no guarantee it is set to the same
/// value type. When this happens we need to define a "sorting order" across types.
///
/// The chosen order is as follows: `Number` < `String` < `bool` < `null`.
/// And within number types the order is: `i64` < `u64` < `f64`.
///
/// ### Handling `NaN`s
///
/// Finally there is one more choice to be made: how do we order `f64::NaN`?
/// By very definition `NaN` can't be ordered or compared, even to itself, but we need
/// a total order for our nodes so they can be selected/processed correctly.
///
/// But our use case does not require an absolute, fixed sort order and instead simply
/// aims for users to tell us which nodes to pick/process "first".
/// There is no need for (mathematical) correctness, just consistent results.
/// Additionally `NaN` as a `Node` attribute feels like a rare and unexpected event.
///
/// For these reasons the `Node` sorting logic assumes that `NaN == NaN` even if this is wrong!
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NodeSearch {
    /// Select nodes that match the given attributes.
    ///
    /// - Nodes are selected if all listed attributes are set and the node value matches.
    /// - The empty set matches all nodes (default).
    #[serde(default)]
    pub matches: NodeSearchMatches,

    /// Limit the number of selected nodes to the first N, or unlimited if `None`.
    #[serde(default)]
    pub max_results: Option<usize>,

    /// Order nodes one or more by attribute.
    ///
    /// The vector lists the sorting attributes by priority.
    /// An optional field prefix indicates ascending (default) or descending order:
    ///
    /// - `+` for ascending.
    /// - `-` for descending.
    ///
    /// ## Default
    ///
    /// By default nodes are sorted ascending by Node ID.
    #[serde(default = "NodeSearch::default_sort_by")]
    pub sort_by: Vec<String>,
}

impl NodeSearch {
    fn default_sort_by() -> Vec<String> {
        vec![String::from("node_id")]
    }
}

impl Default for NodeSearch {
    fn default() -> Self {
        NodeSearch {
            matches: Default::default(),
            max_results: None,
            sort_by: NodeSearch::default_sort_by(),
        }
    }
}

/// Type alias for collection of attribute values to match for node searches.
pub type NodeSearchMatches = HashMap<String, AttributeMatcher>;
