use actix_web::test::call_service;
use actix_web::test::init_service;
use actix_web::test::read_body_json;
use actix_web::test::TestRequest;
use anyhow::Result;

use crate::agent::framework::DefaultContext;
use crate::agent::framework::NodeInfo;
use crate::agent::models::AgentVersion;
use crate::agent::models::Node;
use crate::agent::models::StoreVersion;

use super::into_actix_service;

#[derive(Clone)]
struct FakeAgent {}

impl FakeAgent {
    fn new() -> FakeAgent {
        FakeAgent {}
    }
}

#[async_trait::async_trait]
impl NodeInfo for FakeAgent {
    type Context = DefaultContext;

    async fn node_info(&self, _: &Self::Context) -> Result<Node> {
        Ok(Node {
            agent_version: AgentVersion {
                checkout: "commit".into(),
                number: "1.2.3".into(),
                taint: "for-sure".into(),
            },
            attributes: Default::default(),
            node_id: "id-test-node".into(),
            node_status: crate::agent::models::NodeStatus::Unhealthy,
            store_id: "test.mock".into(),
            store_version: StoreVersion {
                checkout: None,
                number: "3.2.1".into(),
                extra: None,
            },
        })
    }
}

#[tokio::test]
async fn info_node() {
    let logger = slog::Logger::root(slog::Discard {}, slog::o!());
    let agent = into_actix_service(FakeAgent::new(), logger);
    let app = actix_web::App::new().service(agent);
    let req = TestRequest::get().uri("/info/node").to_request();

    let app = init_service(app).await;
    let res = call_service(&app, req).await;
    assert_eq!(res.status(), actix_web::http::StatusCode::OK);

    let node: Node = read_body_json(res).await;
    assert_eq!(node.node_id, "id-test-node");
    assert_eq!(node.store_id, "test.mock");
}
