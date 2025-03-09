use std::net::SocketAddr;

use log::info;

mod api;
mod blockchain;

use blockchain::create_shared_blockchain;

#[tokio::main]
async fn main() {
    // Initialize logger
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    // Create a new blockchain with difficulty 4 and mining reward 100
    let blockchain = create_shared_blockchain(4, 100.0);

    // Create the API router
    let app = api::create_router(blockchain);

    // Define the address to run the server on
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    info!("Starting blockchain server on {}", addr);

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
