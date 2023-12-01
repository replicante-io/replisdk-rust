//! An [`actix_web`] error type that works with [`anyhow::Error`].
use std::sync::Arc;

use actix_web::body::BoxBody;
use actix_web::error::ResponseError;
use actix_web::http::StatusCode;
use actix_web::HttpResponse;

/// Short-hand type for custom response rendering functions.
type CustomRenderFn =
    Arc<dyn Fn(StatusCode, &anyhow::Error) -> HttpResponse<BoxBody> + Send + Sync>;

/// Error type to bridging [`anyhow::Error`] to [`actix_web`].
#[derive(Debug, thiserror::Error)]
pub struct Error {
    /// The underlying [`anyhow::Error`] error.
    #[source]
    source: anyhow::Error,

    /// The HTTP status code for the error response.
    status: StatusCode,

    /// Strategy to render the [`Error`] HTTP response.
    response_strategy: ResponseStrategy,
}

impl Error {
    /// Shortcut for [`Error::with_status`] to generate Bad Bad Request responses.
    pub fn bad_request<E>(source: E) -> Self
    where
        E: Into<anyhow::Error>,
    {
        Self::with_status(StatusCode::BAD_REQUEST, source)
    }

    /// Bridge an [`anyhow::Error`] to create responses with a custom status code.
    pub fn with_status<E>(status: StatusCode, source: E) -> Self
    where
        E: Into<anyhow::Error>,
    {
        Self {
            source: source.into(),
            status,
            response_strategy: ResponseStrategy::Json,
        }
    }

    /// Update the response rendering strategy for the error.
    pub fn use_strategy<S>(mut self, strategy: S) -> Self
    where
        S: Into<ResponseStrategy>,
    {
        self.response_strategy = strategy.into();
        self
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let code = self.status.as_u16();
        let reason = self.status.canonical_reason().unwrap_or("<undefined>");
        write!(f, "HTTP {} ({}) error response: ", code, reason)?;
        std::fmt::Display::fmt(&self.source, f)
    }
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        self.status
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        self.response_strategy.render(self)
    }
}

impl From<anyhow::Error> for Error {
    fn from(source: anyhow::Error) -> Self {
        // Start with defaults in case there is no response data to propagate.
        let mut status = StatusCode::INTERNAL_SERVER_ERROR;
        let mut response_strategy = ResponseStrategy::Json;

        // Look for the latest `Error` instance to propagate error response data.
        for nested in source.chain() {
            if let Some(nested) = nested.downcast_ref::<Error>() {
                status = nested.status;
                response_strategy = nested.response_strategy.clone();
                break;
            }
        }

        // Wrap the error while propagating response data.
        Error {
            source,
            status,
            response_strategy,
        }
    }
}

/// Strategies to render [`Error`] HTTP responses.
#[derive(Clone)]
pub enum ResponseStrategy {
    /// Render a response with a custom strategy.
    Custom(CustomRenderFn),

    /// Render a JSON object with error information.
    Json,

    /// Render a JSON object with the provided body.
    JsonWithBody(serde_json::Value),

    /// Render a JSON object with error information, including a backtrace if available.
    JsonWithTrace,
}

impl std::fmt::Debug for ResponseStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Custom(_) => f
                .debug_tuple("Custom")
                .field(&"<Fn(StatusCode, &anyhow::Error) -> HttpResponse<BoxBody>>")
                .finish(),
            Self::Json => write!(f, "Json"),
            Self::JsonWithBody(body) => f.debug_tuple("JsonWithBody").field(body).finish(),
            Self::JsonWithTrace => write!(f, "JsonWithTrace"),
        }
    }
}

impl ResponseStrategy {
    /// Render an HTTP error response based on the [`Error`]'s strategy,
    fn render(&self, error: &Error) -> HttpResponse<BoxBody> {
        match self {
            Self::Custom(strategy) => self.render_custom(strategy, error),
            Self::Json => self.render_json(error, false),
            Self::JsonWithBody(body) => self.render_json_body(error, body),
            Self::JsonWithTrace => self.render_json(error, true),
        }
    }

    fn render_custom(&self, strategy: &CustomRenderFn, error: &Error) -> HttpResponse<BoxBody> {
        strategy(error.status, &error.source)
    }

    /// Render a JSON object with error information.
    ///
    /// In extended mode include:
    ///
    /// - A backtrace, if one is available,
    fn render_json(&self, error: &Error, extended: bool) -> HttpResponse<BoxBody> {
        let status = error.status_code();
        let error_cause = error.source.root_cause().to_string();
        let error_msg = error.source.to_string();
        let error_trail: Vec<String> = error.source.chain().map(ToString::to_string).collect();

        let mut payload = serde_json::Map::new();
        payload.insert("error".into(), true.into());
        if error_msg != error_cause {
            payload.insert("error_cause".into(), error_cause.into());
        }
        payload.insert("error_msg".into(), error_msg.into());
        if error_trail.len() > 2 {
            payload.insert("error_trail".into(), error_trail.into());
        }
        if extended {
            let backtrace = error.source.backtrace().to_string();
            if !backtrace.is_empty() && backtrace != crate::utils::BACKTRACE_DISABLED {
                payload.insert("error_backtrace".into(), backtrace.into());
            }
        }

        let mut response = HttpResponse::build(status);
        response.insert_header((actix_web::http::header::CONTENT_TYPE, "application/json"));
        response.json(serde_json::Value::from(payload))
    }

    /// Render a JSON object with the provided body.
    fn render_json_body(&self, error: &Error, body: &serde_json::Value) -> HttpResponse<BoxBody> {
        let status = error.status_code();
        let mut response = HttpResponse::build(status);
        response.insert_header((actix_web::http::header::CONTENT_TYPE, "application/json"));
        response.json(body)
    }
}

impl From<serde_json::Value> for ResponseStrategy {
    fn from(body: serde_json::Value) -> ResponseStrategy {
        ResponseStrategy::JsonWithBody(body)
    }
}

impl<S> From<S> for ResponseStrategy
where
    S: Fn(StatusCode, &anyhow::Error) -> HttpResponse<BoxBody> + Send + Sync + 'static,
{
    fn from(strategy: S) -> ResponseStrategy {
        ResponseStrategy::Custom(Arc::new(strategy))
    }
}

/// Short-hand for [`std::result::Result`] with error of type [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use actix_web::http::StatusCode;
    use actix_web::ResponseError;

    use super::Error;

    #[derive(Debug, thiserror::Error)]
    #[error("intermediate error to wrap other errors")]
    struct MidErr(#[from] Error);

    fn custom_render(
        status: actix_web::http::StatusCode,
        source: &anyhow::Error,
    ) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        let mut response = actix_web::HttpResponse::build(status);
        let status = status.as_u16();
        let msg = source.to_string();
        response.body(format!("error from custom strategy: {} - {}", status, msg))
    }

    #[actix_web::test]
    async fn from_anyhow() {
        let error = anyhow::anyhow!("test error");
        let error = Error::from(error);
        assert_eq!(error.status_code(), StatusCode::INTERNAL_SERVER_ERROR);

        let body = actix_web::body::to_bytes(error.error_response().into_body())
            .await
            .unwrap();
        assert_eq!(body, "{\"error\":true,\"error_msg\":\"test error\"}");
    }

    #[actix_web::test]
    async fn from_anyhow_with_context_status() {
        let cause = anyhow::anyhow!("root error");
        let cause = Error::with_status(StatusCode::NOT_FOUND, cause);
        let middle = anyhow::anyhow!(cause).context("middle error");
        let middle = Error::with_status(StatusCode::ACCEPTED, middle);
        let error = anyhow::anyhow!(middle).context("test error");
        let error = Error::from(error);
        assert_eq!(error.status_code(), StatusCode::ACCEPTED);

        let body = actix_web::body::to_bytes(error.error_response().into_body())
            .await
            .unwrap();
        assert_eq!(
            body,
            "{\"error\":true,\"error_cause\":\"root error\",\"error_msg\":\"test error\",\"error_trail\":[\"test error\",\"HTTP 202 (Accepted) error response: middle error\",\"middle error\",\"HTTP 404 (Not Found) error response: root error\",\"root error\"]}"
        );
    }

    #[actix_web::test]
    async fn from_anyhow_with_gaps() {
        let cause = anyhow::anyhow!("root error");
        let cause = Error::with_status(StatusCode::NOT_FOUND, cause);
        let middle = MidErr::from(cause);
        let error = anyhow::anyhow!(middle).context("test error");
        let error = Error::from(error);
        assert_eq!(error.status_code(), StatusCode::NOT_FOUND);

        let body = actix_web::body::to_bytes(error.error_response().into_body())
            .await
            .unwrap();
        assert_eq!(
            body,
            "{\"error\":true,\"error_cause\":\"root error\",\"error_msg\":\"test error\",\"error_trail\":[\"test error\",\"intermediate error to wrap other errors\",\"HTTP 404 (Not Found) error response: root error\",\"root error\"]}"
        );
    }

    #[actix_web::test]
    async fn use_custom_strategy() {
        let error = anyhow::anyhow!("test error");
        let error = Error::from(error).use_strategy(custom_render);
        let body = actix_web::body::to_bytes(error.error_response().into_body())
            .await
            .unwrap();
        assert_eq!(body, "error from custom strategy: 500 - test error");
    }

    #[actix_web::test]
    async fn with_status() {
        let error = anyhow::anyhow!("test error");
        let error = Error::with_status(StatusCode::FAILED_DEPENDENCY, error);
        assert_eq!(error.status_code(), StatusCode::FAILED_DEPENDENCY);

        let body = actix_web::body::to_bytes(error.error_response().into_body())
            .await
            .unwrap();
        assert_eq!(body, "{\"error\":true,\"error_msg\":\"test error\"}");
    }
}
