use axum::Router;
use crate::AppState;

pub mod public;
pub mod protected;
pub mod middleware;
pub mod error;

pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(public::routes())
        .merge(protected::routes())
        .fallback(error::handler_404)
}
