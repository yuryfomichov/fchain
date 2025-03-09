mod routes;

#[cfg(test)]
pub(crate) mod test_utils {
    use axum_test::TestServer;
    use fchain::blockchain::create_shared_blockchain;
    use fchain::blockchain::SharedBlockchain;

    /// Creates a test blockchain with predefined settings
    pub fn create_test_blockchain() -> SharedBlockchain {
        // Use a lower difficulty for faster tests
        create_shared_blockchain(1, 50.0)
    }

    /// Creates a test server with the API router
    pub async fn create_test_server() -> TestServer {
        let blockchain = create_test_blockchain();
        let app = fchain::api::create_router(blockchain);
        TestServer::new(app).unwrap()
    }
}
