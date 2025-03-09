use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::blockchain::{Block, BlockchainError, SharedBlockchain};

/// Request to mine a new block
#[derive(Debug, Deserialize, ToSchema)]
pub struct MineBlockRequest {
    /// The address where mining rewards should be sent
    pub miner_address: String,
}

/// Response for a successful block mining
#[derive(Debug, Serialize, ToSchema)]
pub struct MineBlockResponse {
    /// Success message
    pub message: String,
    /// The mined block
    pub block: Block,
}

/// Get all blocks in the chain
#[utoipa::path(
    get,
    path = "/blocks",
    tag = "Blockchain",
    responses(
        (status = 200, description = "List of all blocks in the chain", body = Vec<Block>)
    )
)]
pub async fn get_blocks(State(blockchain): State<SharedBlockchain>) -> Json<Vec<Block>> {
    let blockchain = blockchain.lock().unwrap();
    Json(blockchain.chain.clone())
}

/// Mine a new block
#[utoipa::path(
    post,
    path = "/blocks/mine",
    tag = "Blockchain",
    request_body = MineBlockRequest,
    responses(
        (status = 200, description = "Block mined successfully", body = MineBlockResponse),
        (status = 400, description = "Mining failed", body = ErrorResponse)
    )
)]
pub async fn mine_block(
    State(blockchain): State<SharedBlockchain>,
    Json(request): Json<MineBlockRequest>,
) -> Result<Json<MineBlockResponse>, BlockchainError> {
    let mut blockchain = blockchain.lock().unwrap();
    let block = blockchain.mine_pending_transactions(&request.miner_address)?;

    Ok(Json(MineBlockResponse {
        message: "Block mined successfully".to_string(),
        block,
    }))
}
