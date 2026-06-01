use std::fmt::Debug;

use axum::{http::StatusCode, response::IntoResponse};

pub struct HttpError(Option<Box<dyn Debug>>, StatusCode);

impl HttpError {
    pub fn new<E: Debug + 'static>(error: E, code: StatusCode) -> Self {
        Self(Some(Box::new(error)), code)
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> axum::response::Response {
        if let Some(e) = self.0 {
            tracing::error!("{:?}", e);
        }
        self.1.into_response()
    }
}

impl From<anyhow::Error> for HttpError {
    fn from(value: anyhow::Error) -> Self {
        Self(Some(Box::new(value)), StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl From<StatusCode> for HttpError {
    fn from(value: StatusCode) -> Self {
        Self(None, value)
    }
}
