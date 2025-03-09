use axum::{
    routing::{get, post},
    Router,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::blockchain::SharedBlockchain;

use super::docs::ApiDoc;
use super::handlers::{
    create_wallet, get_blocks, get_pending_transactions, import_wallet, mine_block,
    sign_and_create_transaction, validate_address, validate_chain,
};

/// Creates the API router
pub fn create_router(blockchain: SharedBlockchain) -> Router {
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/blocks", get(get_blocks))
        .route("/transactions", post(sign_and_create_transaction))
        .route("/transactions/pending", get(get_pending_transactions))
        .route("/mine", post(mine_block))
        .route("/chain/validate", get(validate_chain))
        // Wallet endpoints
        .route("/wallet/create", get(create_wallet))
        .route("/wallet/import", post(import_wallet))
        .route("/wallet/validate", post(validate_address))
        .with_state(blockchain)
}
