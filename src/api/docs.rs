use utoipa::OpenApi;

use super::handlers::{
    CreateTransactionRequest, CreateTransactionResponse, MineBlockRequest, MineBlockResponse,
    ValidateChainResponse,
};
use crate::blockchain::{Block, Transaction};

#[derive(OpenApi)]
#[openapi(
    paths(
        super::handlers::get_blocks,
        super::handlers::create_transaction,
        super::handlers::get_pending_transactions,
        super::handlers::mine_block,
        super::handlers::validate_chain,
    ),
    components(
        schemas(
            Block,
            Transaction,
            CreateTransactionRequest,
            CreateTransactionResponse,
            MineBlockRequest,
            MineBlockResponse,
            ValidateChainResponse,
        )
    ),
    tags(
        (name = "Blockchain", description = "Blockchain management endpoints")
    )
)]
pub struct ApiDoc;
