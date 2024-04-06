use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};

use crate::{api_error::ApiError, app_state::AppState};

pub struct Authentication;

#[async_trait]
impl FromRequestParts<AppState> for Authentication {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let Some(authentication) = &state.authentication else {
            return Ok(Authentication);
        };

        let Some(header) = parts.headers.get(AUTHORIZATION) else {
            return Err(ApiError::NoAuthentication);
        };

        match header.to_str() {
            Ok(str) if str == authentication.as_ref() => Ok(Authentication),
            Ok(_) => Err(ApiError::BadAuthentication),
            Err(_) => Err(ApiError::NoAuthentication),
        }
    }
}
