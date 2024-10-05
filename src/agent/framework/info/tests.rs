use actix_web::test::call_service;
use actix_web::test::init_service;
use actix_web::test::read_body_json;
use actix_web::test::TestRequest;
use anyhow::Result;

use crate::agent::framework::tests::actix_app;
use crate::agent::framework::NodeInfo;
use crate::agent::models::AgentVersion;
use crate::agent::models::Node;
use crate::agent::models::Shard;
use crate::agent::models::ShardCommitOffset;
use crate::agent::models::ShardRole;
use crate::agent::models::ShardsInfo;
use crate::agent::models::StoreExtras;
use crate::agent::models::StoreVersion;
use crate::context::Context;

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
    async fn node_info(&self, _: &Context) -> Result<Node> {
        Ok(Node {
            address: Default::default(),
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

    async fn shards(&self, _: &Context) -> Result<ShardsInfo> {
        let shard = Shard {
            commit_offset: ShardCommitOffset::seconds(10),
            lag: None,
            role: ShardRole::Recovering,
            shard_id: "shard-mock".into(),
        };
        Ok(ShardsInfo {
            shards: vec![shard],
        })
    }

    async fn store_info(&self, _: &Context) -> Result<StoreExtras> {
        Ok(StoreExtras {
            cluster_id: "cluster-mock".into(),
            attributes: Default::default(),
        })
    }
}

#[tokio::test]
async fn info_node() {
    let agent = into_actix_service(FakeAgent::new());
    let app = actix_app().service(agent);
    let req = TestRequest::get().uri("/info/node").to_request();

    let app = init_service(app).await;
    let res = call_service(&app, req).await;
    assert_eq!(res.status(), actix_web::http::StatusCode::OK);

    let node: Node = read_body_json(res).await;
    assert_eq!(node.node_id, "id-test-node");
    assert_eq!(node.store_id, "test.mock");
}

#[tokio::test]
async fn info_store() {
    let agent = into_actix_service(FakeAgent::new());
    let app = actix_app().service(agent);
    let req = TestRequest::get().uri("/info/store").to_request();

    let app = init_service(app).await;
    let res = call_service(&app, req).await;
    assert_eq!(res.status(), actix_web::http::StatusCode::OK);

    let store: StoreExtras = read_body_json(res).await;
    assert_eq!(store.cluster_id, "cluster-mock");
}

#[tokio::test]
async fn shards() {
    let agent = into_actix_service(FakeAgent::new());
    let app = actix_app().service(agent);
    let req = TestRequest::get().uri("/info/shards").to_request();

    let app = init_service(app).await;
    let res = call_service(&app, req).await;
    assert_eq!(res.status(), actix_web::http::StatusCode::OK);

    let info: ShardsInfo = read_body_json(res).await;
    assert_eq!(
        info.shards,
        vec![Shard {
            commit_offset: ShardCommitOffset::seconds(10),
            lag: None,
            role: ShardRole::Recovering,
            shard_id: "shard-mock".into(),
        }]
    );
}
