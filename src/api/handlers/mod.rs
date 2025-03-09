pub mod blocks;
pub mod chain;
pub mod common;
pub mod transactions;

// Re-export handlers
pub use blocks::{get_blocks, mine_block, MineBlockRequest, MineBlockResponse};
pub use chain::{validate_chain, ValidateChainResponse};
pub use transactions::{
    create_transaction, get_pending_transactions, CreateTransactionRequest,
    CreateTransactionResponse,
};
