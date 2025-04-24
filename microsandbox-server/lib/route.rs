use axum::Router;

use crate::state::AppState;

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

pub fn create_router(state: AppState) -> Router {
    let router = Router::new();
    router
}
