use axum::Json;
use axum::extract::{FromRequest, Request};
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::error::AppError;

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = AppError;

    fn from_request(
        req: Request,
        state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            let Json(value) = Json::<T>::from_request(req, state)
                .await
                .map_err(|_| AppError::BadRequest("invalid JSON body".into()))?;

            value
                .validate()
                .map_err(|e| AppError::BadRequest(e.to_string()))?;

            Ok(ValidatedJson(value))
        }
    }
}
