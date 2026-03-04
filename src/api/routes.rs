use crate::AppState;
use axum::Router;

pub mod error;
pub mod protected;
pub mod public;


pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(public::routes())
        .merge(protected::routes())
        .fallback(error::handler_404)
}
