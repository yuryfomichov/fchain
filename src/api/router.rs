use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use super::docs::ApiDoc;
use super::handlers;
use crate::blockchain::SharedBlockchain;

/// Creates the API router
pub fn create_router(blockchain: SharedBlockchain) -> Router {
    // Configure CORS middleware
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/blocks", get(handlers::get_blocks))
        .route("/blocks/mine", post(handlers::mine_block))
        .route("/transactions", post(handlers::create_transaction))
        .route(
            "/transactions/pending",
            get(handlers::get_pending_transactions),
        )
        .route("/chain/validate", get(handlers::validate_chain))
        .with_state(blockchain)
        .layer(cors)
}
