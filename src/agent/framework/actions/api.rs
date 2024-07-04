//! Action API endpoints.
use actix_web::dev::AppService;
use actix_web::dev::HttpServiceFactory;
use actix_web::web::Data;
use actix_web::web::Path;
use actix_web::HttpResponse;
use actix_web::Responder;

use crate::agent::framework::actions::ActionsRegistry;
use crate::agent::framework::store;
use crate::agent::framework::Injector;
use crate::agent::models::ActionExecution;
use crate::agent::models::ActionExecutionRequest;
use crate::agent::models::ActionExecutionResponse;
use crate::context::Context;
use crate::utils::actix::error::Error;
use crate::utils::actix::error::Result;

/// Register actions API endpoints as an [`actix_web`] service.
#[derive(Clone, Debug)]
pub struct ActionsService {
    /// Catalogue of known action handlers.
    actions: ActionsRegistry,

    /// Interface to the agent persisted store.
    store: store::Store,
}

impl ActionsService {
    /// Initialise an [`ActionsService`] with dependencies from the given [`Injector`].
    pub fn with_injector(injector: &Injector) -> ActionsService {
        ActionsService {
            actions: injector.actions.clone(),
            store: injector.store.clone(),
        }
    }
}

impl HttpServiceFactory for ActionsService {
    fn register(self, config: &mut AppService) {
        let service = self.clone();
        actix_web::web::scope("/actions")
            .app_data(Data::new(service.clone()))
            .service(
                actix_web::web::resource("/finished")
                    .guard(actix_web::guard::Get())
                    .to(finished),
            )
            .service(
                actix_web::web::resource("/queue")
                    .guard(actix_web::guard::Get())
                    .to(queue),
            )
            .register(config);
        actix_web::web::scope("/action")
            .app_data(Data::new(service))
            .service(
                actix_web::web::resource("/{action_id}")
                    .guard(actix_web::guard::Get())
                    .to(lookup),
            )
            .service(
                actix_web::web::resource("")
                    .guard(actix_web::guard::Post())
                    .to(schedule),
            )
            .register(config)
    }
}

/// Query already finished agent actions.
pub async fn finished(service: Data<ActionsService>, context: Context) -> Result<impl Responder> {
    let query = store::query::ActionsFinished {};
    let response = service.store.query(&context, query).await?;
    Ok(HttpResponse::Ok().json(response))
}

pub async fn lookup(
    service: Data<ActionsService>,
    context: Context,
    id: Path<uuid::Uuid>,
) -> Result<impl Responder> {
    let query = store::query::Action::new(id.into_inner());
    let response = service.store.query(&context, query).await?;
    let response = match response {
        None => HttpResponse::NotFound().finish(),
        Some(response) => HttpResponse::Ok().json(response),
    };
    Ok(response)
}

/// Query currently running and queued agent actions.
pub async fn queue(service: Data<ActionsService>, context: Context) -> Result<impl Responder> {
    let query = store::query::ActionsQueue {};
    let response = service.store.query(&context, query).await?;
    Ok(HttpResponse::Ok().json(response))
}

/// Schedule a new action to run on the agent.
pub async fn schedule(
    service: Data<ActionsService>,
    context: Context,
    action: actix_web::web::Json<ActionExecutionRequest>,
) -> Result<impl Responder> {
    // Validate request parameters.
    //  -> Check action kind is known.
    service
        .actions
        .lookup(&action.kind)
        .map_err(|error| Error::with_status(actix_web::http::StatusCode::BAD_REQUEST, error))?;
    //  -> Check created time is in UTC.
    if let Some(created_time) = &action.created_time {
        if !created_time.offset().is_utc() {
            let error = anyhow::anyhow!("The provided created_time MUST be in UTC");
            let error = Error::with_status(actix_web::http::StatusCode::BAD_REQUEST, error);
            return Err(error);
        }
    }

    // Reject scheduling requests with an ID we already know.
    if let Some(action_id) = action.id {
        let query = store::query::Action::new(action_id);
        let known = service.store.query(&context, query).await?;
        if known.is_some() {
            let error = anyhow::anyhow!("The action ID is already used by another action");
            let error = Error::with_status(actix_web::http::StatusCode::CONFLICT, error);
            return Err(error);
        }
    }

    // Store the action in the DB.
    let action = ActionExecution::from(action.into_inner());
    let id = action.id;
    service.store.persist(&context, action).await?;
    Ok(HttpResponse::Ok().json(ActionExecutionResponse { id }))
}

#[cfg(test)]
mod tests {
    use actix_web::test::call_service;
    use actix_web::test::init_service;
    use actix_web::test::read_body_json;
    use actix_web::test::TestRequest;

    use super::ActionsService;
    use crate::agent::framework::tests::actix_app;
    use crate::agent::framework::Injector;
    use crate::agent::models::ActionExecution;
    use crate::agent::models::ActionExecutionList;
    use crate::agent::models::ActionExecutionRequest;
    use crate::agent::models::ActionExecutionResponse;

    fn actions_service(injector: &Injector) -> ActionsService {
        ActionsService::with_injector(injector)
    }

    #[tokio::test]
    async fn finished_actions() {
        let injector = Injector::fixture().await;
        let service = actions_service(&injector);
        let app = actix_app().service(service);
        let app = init_service(app).await;

        let mut action = super::store::fixtures::action(uuid::Uuid::new_v4());
        action.finished_time = Some(time::OffsetDateTime::now_utc());
        let context = super::Context::fixture();
        injector.store.persist(&context, action).await.unwrap();

        let request = TestRequest::get().uri("/actions/finished").to_request();
        let response = call_service(&app, request).await;

        assert_eq!(response.status(), actix_web::http::StatusCode::OK);
        let body: ActionExecutionList = read_body_json(response).await;
        assert_eq!(body.actions.len(), 1);
    }

    #[tokio::test]
    async fn lookup_action() {
        let injector = Injector::fixture().await;
        let id = uuid::Uuid::new_v4();
        let action = super::store::fixtures::action(id);
        let context = super::Context::fixture();
        injector
            .store
            .persist(&context, action.clone())
            .await
            .unwrap();

        let service = actions_service(&injector);
        let app = actix_app().service(service);
        let app = init_service(app).await;

        let request = TestRequest::get()
            .uri(&format!("/action/{}", id))
            .to_request();
        let response = call_service(&app, request).await;

        assert_eq!(response.status(), actix_web::http::StatusCode::OK);
        let body: ActionExecution = read_body_json(response).await;
        assert_eq!(body, action);
    }

    #[tokio::test]
    async fn lookup_action_not_found() {
        let injector = Injector::fixture().await;
        let id = uuid::Uuid::new_v4();

        let service = actions_service(&injector);
        let app = actix_app().service(service);
        let app = init_service(app).await;

        let request = TestRequest::get()
            .uri(&format!("/action/{}", id))
            .to_request();
        let response = call_service(&app, request).await;
        assert_eq!(response.status(), actix_web::http::StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn queued_actions() {
        let injector = Injector::fixture().await;
        let service = actions_service(&injector);
        let app = actix_app().service(service);
        let app = init_service(app).await;

        let action = super::store::fixtures::action(uuid::Uuid::new_v4());
        let context = super::Context::fixture();
        injector.store.persist(&context, action).await.unwrap();

        let request = TestRequest::get().uri("/actions/queue").to_request();
        let response = call_service(&app, request).await;

        assert_eq!(response.status(), actix_web::http::StatusCode::OK);
        let body: ActionExecutionList = read_body_json(response).await;
        assert_eq!(body.actions.len(), 1);
    }

    #[tokio::test]
    async fn schedule_action() {
        let injector = Injector::fixture().await;
        let service = actions_service(&injector);
        let app = actix_app().service(service);
        let app = init_service(app).await;

        let id = uuid::Uuid::new_v4();
        let request = ActionExecutionRequest {
            args: Default::default(),
            created_time: None,
            id: Some(id),
            kind: super::store::fixtures::ACTION_KIND.to_string(),
            metadata: Default::default(),
        };
        let request = TestRequest::post()
            .uri("/action")
            .set_json(request)
            .to_request();
        let response = call_service(&app, request).await;

        assert_eq!(response.status(), actix_web::http::StatusCode::OK);
        let body: ActionExecutionResponse = read_body_json(response).await;
        assert_eq!(body.id, id);
    }

    #[tokio::test]
    async fn schedule_action_created_in_utc() {
        let injector = Injector::fixture().await;
        let service = actions_service(&injector);
        let app = actix_app().service(service);
        let app = init_service(app).await;

        let created_time =
            time::OffsetDateTime::now_utc().to_offset(time::UtcOffset::from_hms(3, 0, 0).unwrap());
        let request = ActionExecutionRequest {
            args: Default::default(),
            created_time: Some(created_time),
            id: None,
            kind: super::store::fixtures::ACTION_KIND.to_string(),
            metadata: Default::default(),
        };
        let request = TestRequest::post()
            .uri("/action")
            .set_json(request)
            .to_request();
        let response = call_service(&app, request).await;

        assert_eq!(response.status(), actix_web::http::StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn schedule_action_kind_not_known() {
        let injector = Injector::fixture().await;
        let service = actions_service(&injector);
        let app = actix_app().service(service);
        let app = init_service(app).await;

        let request = ActionExecutionRequest {
            args: Default::default(),
            created_time: None,
            id: None,
            kind: "not.a/real.action".to_string(),
            metadata: Default::default(),
        };
        let request = TestRequest::post()
            .uri("/action")
            .set_json(request)
            .to_request();
        let response = call_service(&app, request).await;

        assert_eq!(response.status(), actix_web::http::StatusCode::BAD_REQUEST);
    }
}
