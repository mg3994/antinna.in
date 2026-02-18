use salvo::http::{ParseError, StatusCode, StatusError};
use salvo::oapi::{self, EndpointOutRegister, ToSchema};
use salvo::prelude::*;
use thiserror::Error;
use crate::infrastructure::i18n::i18n; // <-- your OnceLock accessor

use salvo::http::header::ACCEPT_LANGUAGE;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("public: `{0}`")]
    Public(String),
    #[error("internal: `{0}`")]
    Internal(String),
    #[error("unauthorized: `{0}`")]
    Unauthorized(String),
    #[error("forbidden: `{0}`")]
    Forbidden(String),
    #[error("salvo internal error: `{0}`")]
    Salvo(#[from] ::salvo::Error),
    #[error("http status error: `{0}`")]
    HttpStatus(#[from] StatusError),
    #[error("http parse error:`{0}`")]
    HttpParse(#[from] ParseError),
    #[error("anyhow error:`{0}`")]
    Anyhow(#[from] anyhow::Error),
    #[error("sqlx::Error:`{0}`")]
    SqlxError(#[from] sqlx::Error),
    #[error("validation error:`{0}`")]
    Validation(#[from] validator::ValidationErrors),
}
impl AppError {
    pub fn public<S: Into<String>>(msg: S) -> Self {
        Self::Public(msg.into())
    }

    pub fn internal<S: Into<String>>(msg: S) -> Self {
        Self::Internal(msg.into())
    }

    // Added helper methods
    pub fn unauthorized<S: Into<String>>(msg: S) -> Self { Self::Unauthorized(msg.into()) }
    pub fn forbidden<S: Into<String>>(msg: S) -> Self { Self::Forbidden(msg.into()) }
}

#[async_trait]
impl Writer for AppError {
    async fn write(mut self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        let (code, key) = match &self {
            Self::Unauthorized(_) => (StatusCode::UNAUTHORIZED, "errors.unauthorized"),
            Self::Forbidden(_) => (StatusCode::FORBIDDEN, "errors.forbidden"),
            Self::HttpStatus(e) => (e.code, "errors.bad_request"),
            Self::Validation(_) => (StatusCode::BAD_REQUEST, "errors.validation_error"),
            Self::SqlxError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "database.query_failed"),
            Self::Salvo(_) => (StatusCode::INTERNAL_SERVER_ERROR, "errors.salvo_error"),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "errors.internal_error"),
        };
        res.status_code(code);

        // 2. Extract language safely
        // ------------------------
        let default_locale = i18n().default_locale().to_string();

        let lang = _req
            .header::<String>(ACCEPT_LANGUAGE)
            .unwrap_or(default_locale.clone());

        let lang = lang
            .split(',')
            .next()
            .unwrap_or(&default_locale)
            .split('-')
            .next()
            .unwrap_or(&default_locale);


        // 3. Translate message
        // ------------------------
        let message = match self {
            // Public errors pass through
            Self::Public(msg) => msg,

            Self::Unauthorized(msg) => {
                tracing::warn!("unauthorized access attempt: {}", msg);
                i18n().translate(key, lang)
            }
            Self::Forbidden(msg) => {
                tracing::warn!("forbidden access attempt: {}", msg);
                i18n().translate(key, lang)
            }

            // Internal errors logged but localized
            Self::Internal(msg) => {
                tracing::error!("internal error: {}", msg);
                i18n().translate(key, lang)
            }

            Self::Salvo(e) => {
                tracing::error!(error = ?e, "salvo error");
                i18n().translate(key, lang)
            }

            Self::SqlxError(e) => {
                tracing::error!(error = ?e, "database error");
                i18n().translate(key, lang)
            }

            Self::Validation(e) => {
                tracing::warn!(error = ?e, "validation error");
                i18n().translate(key, lang)
            }

            Self::HttpStatus(e) => {
                tracing::warn!(error = ?e, "http status error");
                i18n().translate(key, lang)
            }

            e => {
                tracing::error!("unknown error: {}", e);
                i18n().translate("errors.unknown_error", lang)
            }
        };

        // ------------------------
        // 4. Render localized error
        // ------------------------
        let err = StatusError::from_code(code)
            .unwrap_or_else(StatusError::internal_server_error)
            .brief(message);

        res.render(err);
    }
}
impl EndpointOutRegister for AppError {
    fn register(
        components: &mut salvo::oapi::Components,
        operation: &mut salvo::oapi::Operation,
    ) {
        let schema = StatusError::to_schema(components);

        operation.responses.insert(
            StatusCode::INTERNAL_SERVER_ERROR.as_str(),
            oapi::Response::new("Internal server error")
                .add_content("application/json", schema.clone()),
        );

        operation.responses.insert(
            StatusCode::BAD_REQUEST.as_str(),
            oapi::Response::new("Bad request / Validation error")
                .add_content("application/json", schema.clone()),
        );

        operation.responses.insert(
            StatusCode::NOT_FOUND.as_str(),
            oapi::Response::new("Resource not found")
                .add_content("application/json", schema.clone()),
        );

        operation.responses.insert(
            StatusCode::UNAUTHORIZED.as_str(),
            oapi::Response::new("Unauthorized")
                .add_content("application/json", schema.clone()),
        );

        operation.responses.insert(
            StatusCode::FORBIDDEN.as_str(),
            oapi::Response::new("Forbidden")
                .add_content("application/json", schema.clone()),
        );

        operation.responses.insert(
            StatusCode::CONFLICT.as_str(),
            oapi::Response::new("Conflict")
                .add_content("application/json", schema),
        );
    }
}
