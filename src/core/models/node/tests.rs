//! Unit tests for node models.
use super::AttributeValueRef;
use super::Node;
use super::NodeDetails;
use super::NodeStatus;

#[rstest::rstest]
#[case("ns_id", Some("test-ns"))]
#[case("cluster_id", Some("test-cluster"))]
#[case("node_id", Some("test-node"))]
#[case("node_status", Some("INCOMPLETE"))]
#[case("agent_version", None)]
fn node_attribute_lookup(#[case] attribute: &str, #[case] expected: Option<&str>) {
    let node = Node {
        ns_id: "test-ns".into(),
        cluster_id: "test-cluster".into(),
        node_id: "test-node".into(),
        details: None,
        node_status: NodeStatus::Incomplete,
    };
    let actual = node.attribute(attribute);
    let actual = actual.map(|actual| match actual {
        AttributeValueRef::String(actual) => actual,
        _ => panic!("test requires a string attribute"),
    });
    assert_eq!(actual, expected);
}

#[rstest::rstest]
#[case("agent_version", Some("1.2.3"))]
#[case("agent_version.checkout", Some("agent-sha"))]
#[case("agent_version.number", Some("1.2.3"))]
#[case("agent_version.taint", Some("test"))]
#[case("store_id", Some("test-store"))]
#[case("store_version", Some("4.5.6"))]
#[case("store_version.checkout", None)]
#[case("store_version.number", Some("4.5.6"))]
#[case("store_version.extra", Some("mocked"))]
#[case("test.attribute", Some("value"))]
#[case("missing-attribute", None)]
fn node_attribute_lookup_details(#[case] attribute: &str, #[case] expected: Option<&str>) {
    let details = NodeDetails {
        address: Default::default(),
        agent_version: super::AgentVersion {
            checkout: "agent-sha".into(),
            number: "1.2.3".into(),
            taint: "test".into(),
        },
        attributes: {
            let mut map = std::collections::BTreeMap::new();
            map.insert("test.attribute".into(), "value".into());
            map
        },
        store_id: "test-store".into(),
        store_version: super::StoreVersion {
            checkout: None,
            number: "4.5.6".into(),
            extra: Some("mocked".into()),
        },
    };
    let node = Node {
        ns_id: "test-ns".into(),
        cluster_id: "test-cluster".into(),
        node_id: "test-node".into(),
        details: Some(details),
        node_status: NodeStatus::Unhealthy,
    };
    let actual = node.attribute(attribute);
    let actual = actual.map(|actual| match actual {
        AttributeValueRef::String(actual) => actual,
        _ => panic!("test requires a string attribute"),
    });
    assert_eq!(actual, expected);
}
