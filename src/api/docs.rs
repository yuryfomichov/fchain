use utoipa::OpenApi;

use crate::api::handlers::{
    CreateTransactionRequest, CreateTransactionResponse, MineBlockRequest, MineBlockResponse,
    ValidateChainResponse,
};
use crate::blockchain::crypto::{Address, PublicKeyHex, TransactionSignature};
use crate::blockchain::{Block, Transaction};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::handlers::blocks::get_blocks,
        crate::api::handlers::transactions::create_transaction,
        crate::api::handlers::transactions::get_pending_transactions,
        crate::api::handlers::blocks::mine_block,
        crate::api::handlers::chain::validate_chain,
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
            Address,
            PublicKeyHex,
            TransactionSignature,
        )
    ),
    tags(
        (name = "Blockchain", description = "Blockchain management endpoints")
    )
)]
pub struct ApiDoc;
