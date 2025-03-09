// Re-export modules for testing and library usage
pub mod api;
pub mod blockchain;

// Re-export main types for convenience
pub use blockchain::block::Block;
pub use blockchain::blockchain::{Blockchain, BlockchainError, SharedBlockchain};
pub use blockchain::transaction::Transaction;
