//! Action API endpoints.
use actix_web::dev::AppService;
use actix_web::dev::HttpServiceFactory;
use actix_web::web::Data;
use actix_web::HttpResponse;
use actix_web::Responder;

use crate::agent::framework::store;
use crate::agent::framework::DefaultContext;
use crate::agent::framework::Injector;
use crate::agent::models::ActionExecution;
use crate::agent::models::ActionExecutionRequest;
use crate::agent::models::ActionExecutionResponse;
use crate::utils::actix::error::Result;

/// Register actions API endpoints as an [`actix_web`] service.
#[derive(Clone, Debug)]
pub struct ActionsService {
    /// The [`slog::Logger`] usable to make [`DefaultContext`](super::DefaultContext) instances.
    logger: slog::Logger,

    /// Interface to the agent persisted store.
    store: store::Store,
}

impl ActionsService {
    /// Build an [`actix_web`] service factory to handle actions API requests.
    pub fn build(logger: slog::Logger) -> ActionsServiceBuilder {
        ActionsServiceBuilder { logger }
    }
}

impl HttpServiceFactory for ActionsService {
    fn register(self, config: &mut AppService) {
        let service = self.clone();
        actix_web::web::scope("/actions")
            .app_data(Data::new(service.clone()))
            .app_data(Data::new(self.logger.clone()))
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
            .app_data(Data::new(self.logger))
            //.service(
            //    actix_web::web::resource("/{action_id}")
            //        .guard(actix_web::guard::Get())
            //        .to(node::info_node::<I>),
            //)
            .service(
                actix_web::web::resource("")
                    .guard(actix_web::guard::Post())
                    .to(schedule),
            )
            .register(config)
    }
}

pub struct ActionsServiceBuilder {
    // The [`slog::Logger`] usable to make [`DefaultContext`](super::DefaultContext) instances.
    logger: slog::Logger,
}

impl ActionsServiceBuilder {
    pub fn with_injector(self, injector: &Injector) -> ActionsService {
        ActionsService {
            logger: self.logger,
            store: injector.store.clone(),
        }
    }
}

/// Query already finished agent actions.
pub async fn finished(
    service: Data<ActionsService>,
    context: DefaultContext,
) -> Result<impl Responder> {
    let query = store::query::ActionsFinished {};
    let response = service.store.query(&context, query).await?;
    Ok(HttpResponse::Ok().json(response))
}

/// Query currently running and queued agent actions.
pub async fn queue(
    service: Data<ActionsService>,
    context: DefaultContext,
) -> Result<impl Responder> {
    let query = store::query::ActionsQueue {};
    let response = service.store.query(&context, query).await?;
    Ok(HttpResponse::Ok().json(response))
}

/// Schedule a new action to run on the agent.
pub async fn schedule(
    service: Data<ActionsService>,
    context: DefaultContext,
    action: actix_web::web::Json<ActionExecutionRequest>,
) -> Result<impl Responder> {
    // Validate request parameters.
    // TODO: -> Check action kind is known.
    //  -> Check created time is in UTC.
    if let Some(created_time) = &action.created_time {
        if !created_time.offset().is_utc() {
            let error = anyhow::anyhow!("The provided created_time MUST be in UTC");
            let error = crate::utils::actix::error::Error::with_status(
                actix_web::http::StatusCode::BAD_REQUEST,
                error,
            );
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
    use crate::agent::framework::Injector;
    use crate::agent::models::ActionExecutionList;
    use crate::agent::models::ActionExecutionRequest;
    use crate::agent::models::ActionExecutionResponse;

    fn actions_service(injector: &Injector) -> ActionsService {
        let logger = slog::Logger::root(slog::Discard, slog::o!());
        ActionsService::build(logger).with_injector(injector)
    }

    #[tokio::test]
    async fn finished_actions() {
        let injector = Injector::fixture().await;
        let service = actions_service(&injector);
        let app = actix_web::App::new().service(service);
        let app = init_service(app).await;

        let mut action = super::store::fixtures::action(uuid::Uuid::new_v4());
        action.finished_time = Some(time::OffsetDateTime::now_utc());
        let context = super::DefaultContext::fixture();
        injector.store.persist(&context, action).await.unwrap();

        let request = TestRequest::get().uri("/actions/finished").to_request();
        let response = call_service(&app, request).await;

        assert_eq!(response.status(), actix_web::http::StatusCode::OK);
        let body: ActionExecutionList = read_body_json(response).await;
        assert_eq!(body.actions.len(), 1);
    }

    #[tokio::test]
    async fn queued_actions() {
        let injector = Injector::fixture().await;
        let service = actions_service(&injector);
        let app = actix_web::App::new().service(service);
        let app = init_service(app).await;

        let action = super::store::fixtures::action(uuid::Uuid::new_v4());
        let context = super::DefaultContext::fixture();
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
        let app = actix_web::App::new().service(service);
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
        let app = actix_web::App::new().service(service);
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

    // TODO: schedule_action_kind_not_known
}
