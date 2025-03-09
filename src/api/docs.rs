use utoipa::OpenApi;

use super::handlers::{
    CreateTransactionResponse, CreateWalletResponse, ImportWalletRequest, ImportWalletResponse,
    MineBlockRequest, MineBlockResponse, SignAndCreateTransactionRequest, ValidateAddressRequest,
    ValidateAddressResponse, ValidateChainResponse,
};
use crate::blockchain::{Block, Transaction};

#[derive(OpenApi)]
#[openapi(
    paths(
        super::handlers::get_blocks,
        super::handlers::sign_and_create_transaction,
        super::handlers::get_pending_transactions,
        super::handlers::mine_block,
        super::handlers::validate_chain,
        super::handlers::create_wallet,
        super::handlers::import_wallet,
        super::handlers::validate_address,
    ),
    components(
        schemas(
            Block,
            Transaction,
            SignAndCreateTransactionRequest,
            CreateTransactionResponse,
            MineBlockRequest,
            MineBlockResponse,
            ValidateChainResponse,
            CreateWalletResponse,
            ImportWalletRequest,
            ImportWalletResponse,
            ValidateAddressRequest,
            ValidateAddressResponse,
        )
    ),
    tags(
        (name = "Blockchain", description = "Blockchain management endpoints"),
        (name = "Wallet", description = "Wallet management endpoints")
    )
)]
pub struct ApiDoc;
