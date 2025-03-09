use axum::{
    routing::{get, post},
    Router,
};

use crate::blockchain::SharedBlockchain;

use super::handlers::{
    create_transaction, get_blocks, get_pending_transactions, mine_block, validate_chain,
};

/// Creates the API router
pub fn create_router(blockchain: SharedBlockchain) -> Router {
    Router::new()
        .route("/blocks", get(get_blocks))
        .route("/transactions", post(create_transaction))
        .route("/transactions/pending", get(get_pending_transactions))
        .route("/mine", post(mine_block))
        .route("/chain/validate", get(validate_chain))
        .with_state(blockchain)
}
