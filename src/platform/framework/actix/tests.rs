use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use actix_web::test::call_service;
use actix_web::test::init_service;
use actix_web::test::read_body_json;
use actix_web::test::TestRequest;
use anyhow::Result;

use crate::platform::framework::DefaultContext;
use crate::platform::framework::IPlatform;
use crate::platform::models::ClusterDiscovery;
use crate::platform::models::ClusterDiscoveryNode;
use crate::platform::models::ClusterDiscoveryResponse;
use crate::platform::models::NodeDeprovisionRequest;
use crate::platform::models::NodeProvisionRequest;
use crate::platform::models::NodeProvisionResponse;

use super::into_actix_service;

struct FakePlatform {
    deprovision_called: Arc<AtomicBool>,
}

impl FakePlatform {
    fn new() -> FakePlatform {
        FakePlatform {
            deprovision_called: Arc::new(AtomicBool::new(false)),
        }
    }
}

#[async_trait::async_trait]
impl IPlatform for FakePlatform {
    type Context = DefaultContext;

    async fn deprovision(&self, _: &Self::Context, _request: NodeDeprovisionRequest) -> Result<()> {
        self.deprovision_called.store(true, Ordering::SeqCst);
        Ok(())
    }

    async fn discover(&self, _: &Self::Context) -> Result<ClusterDiscoveryResponse> {
        let cluster_a = ClusterDiscovery {
            cluster_id: "a".into(),
            nodes: vec![
                ClusterDiscoveryNode {
                    agent_address: "http://1.a:2345".into(),
                    node_class: "mock".into(),
                    node_id: "1".into(),
                    node_group: None,
                },
                ClusterDiscoveryNode {
                    agent_address: "http://2.a:2345".into(),
                    node_class: "mock".into(),
                    node_id: "2".into(),
                    node_group: None,
                },
            ],
        };
        let cluster_b = ClusterDiscovery {
            cluster_id: "b".into(),
            nodes: vec![
                ClusterDiscoveryNode {
                    agent_address: "http://1.b:2345".into(),
                    node_class: "mock".into(),
                    node_id: "1".into(),
                    node_group: None,
                },
                ClusterDiscoveryNode {
                    agent_address: "http://2.b:2345".into(),
                    node_class: "mock".into(),
                    node_id: "2".into(),
                    node_group: None,
                },
            ],
        };
        Ok(ClusterDiscoveryResponse {
            clusters: vec![cluster_a, cluster_b],
        })
    }

    async fn provision(
        &self,
        _: &Self::Context,
        _: NodeProvisionRequest,
    ) -> Result<NodeProvisionResponse> {
        Ok(NodeProvisionResponse {
            count: 2,
            node_ids: None,
        })
    }
}

#[tokio::test]
async fn deprovision() {
    let platform = FakePlatform::new();
    let deprovision = Arc::clone(&platform.deprovision_called);

    let logger = slog::Logger::root(slog::Discard {}, slog::o!());
    let platform = into_actix_service(platform, logger);
    let app = actix_web::App::new().service(platform);

    let payload = r#"{"cluster_id": "c", "node_id": "n"}"#.as_bytes();
    let req = TestRequest::post()
        .uri("/deprovision")
        .insert_header((actix_web::http::header::CONTENT_TYPE, "application/json"))
        .set_payload(payload)
        .to_request();

    let app = init_service(app).await;
    let res = call_service(&app, req).await;
    assert_eq!(res.status(), actix_web::http::StatusCode::NO_CONTENT);
    assert!(deprovision.load(Ordering::SeqCst));
}

#[tokio::test]
async fn discover() {
    let logger = slog::Logger::root(slog::Discard {}, slog::o!());
    let platform = into_actix_service(FakePlatform::new(), logger);
    let app = actix_web::App::new().service(platform);

    let req = TestRequest::get().uri("/discover").to_request();

    let app = init_service(app).await;
    let res = call_service(&app, req).await;
    assert_eq!(res.status(), actix_web::http::StatusCode::OK);

    let res: ClusterDiscoveryResponse = read_body_json(res).await;
    assert_eq!(res.clusters.len(), 2);
}

#[tokio::test]
async fn provision() {
    let logger = slog::Logger::root(slog::Discard {}, slog::o!());
    let platform = into_actix_service(FakePlatform::new(), logger);
    let app = actix_web::App::new().service(platform);

    let payload = r#"{
"cluster": {
    "cluster_id": "a",
    "store": "test",
    "store_version": "1",
    "nodes": {
        "default": {
            "desired_count": 10,
            "node_class": "test"
        }
    }
},
"provision": {
    "node_group_id": "default"
}
    }"#
    .as_bytes();
    let req = TestRequest::post()
        .uri("/provision")
        .insert_header((actix_web::http::header::CONTENT_TYPE, "application/json"))
        .set_payload(payload)
        .to_request();

    let app = init_service(app).await;
    let res = call_service(&app, req).await;
    assert_eq!(res.status(), actix_web::http::StatusCode::OK);

    let res: NodeProvisionResponse = read_body_json(res).await;
    assert_eq!(res.count, 2);
}

#[tokio::test]
async fn platform_is_wrapped_in_app() {
    let logger = slog::Logger::root(slog::Discard {}, slog::o!());
    let platform = into_actix_service(FakePlatform::new(), logger);
    let _ = actix_web::App::new().service(platform);
}

fn node_provision_request<S: Into<String>>(group: S) -> NodeProvisionRequest {
    NodeProvisionRequest {
        cluster: crate::platform::models::ClusterDefinition {
            attributes: Default::default(),
            cluster_id: "test.cluster".into(),
            store: "noop".into(),
            store_version: "0.0.0".into(),
            nodes: {
                let mut nodes = std::collections::HashMap::new();
                nodes.insert(
                    "default".into(),
                    crate::platform::models::ClusterDefinitionNodeGroup {
                        attributes: Default::default(),
                        desired_count: 3,
                        node_class: "test".into(),
                        store_version: None,
                    },
                );
                nodes
            },
        },
        provision: crate::platform::models::NodeProvisionRequestDetails {
            node_group_id: group.into(),
        },
    }
}

mod resolve_node_group_clone {
    use super::super::NodeProvisionRequestExt;

    #[tokio::test]
    async fn found() {
        let request = super::node_provision_request("default");
        let group = request.resolve_node_group_clone().unwrap();
        assert_eq!(group.desired_count, 3);
        assert_eq!(group.node_class, "test");

        // Ensure the group is still defined.
        let group = request.resolve_node_group_clone().unwrap();
        assert_eq!(group.desired_count, 3);
        assert_eq!(group.node_class, "test");
    }

    #[tokio::test]
    #[should_panic]
    async fn not_found() {
        let request = super::node_provision_request("not-default");
        let _ = request.resolve_node_group_clone().unwrap();
    }
}

mod resolve_node_group_remove {
    use super::super::NodeProvisionRequestExt;

    #[tokio::test]
    async fn found() {
        let mut request = super::node_provision_request("default");
        let group = request.resolve_node_group_remove().unwrap();
        assert_eq!(group.desired_count, 3);
        assert_eq!(group.node_class, "test");

        // Ensure the group is gone.
        let group = request.resolve_node_group_remove();
        assert_eq!(group.is_err(), true);
    }

    #[tokio::test]
    #[should_panic]
    async fn not_found() {
        let mut request = super::node_provision_request("not-default");
        let _ = request.resolve_node_group_remove().unwrap();
    }
}
