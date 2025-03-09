# FChain - A Simple Blockchain in Rust

![Rust CI](https://github.com/yuryfomichov/fchain/workflows/Rust%20CI/badge.svg)

This project implements a basic blockchain with the following features:
- Core blockchain data structures (Block, Transaction, Blockchain)
- Proof of Work consensus mechanism
- HTTP API for interacting with the blockchain

## Development

This project uses Dev Containers for a consistent development environment. To get started:

1. Install [Docker](https://www.docker.com/products/docker-desktop) and [VS Code](https://code.visualstudio.com/)
2. Install the [Remote - Containers](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers) extension for VS Code
3. Clone this repository
4. Open the project in VS Code and click "Reopen in Container" when prompted
5. The container will build with Rust 1.85 and all necessary dependencies

## Project Structure

```
fchain/
├── src/
│   ├── blockchain/    # Core blockchain implementation
│   ├── api/           # HTTP API
│   └── main.rs        # Application entry point
└── tests/             # Integration tests
```

## Running the Application

```bash
# Run the application
cargo run

# Run with logging
RUST_LOG=info cargo run

# Run tests
cargo test
```

## Continuous Integration

This project uses GitHub Actions for continuous integration. The CI pipeline automatically runs on every push to the main branch and on pull requests.

The CI pipeline performs the following checks:
- Code formatting (rustfmt)
- Linting (clippy)
- Building the project
- Running all tests

You can see the status of the CI pipeline in the GitHub repository under the "Actions" tab.

## Accessing the Application

The application exposes two ports:
- **Port 3000**: The blockchain API server
- **Port 8000**: Documentation server

### API Documentation
The API is documented using OpenAPI (Swagger) specification. You can access the interactive API documentation in two ways:

1. **Swagger UI** (when the application is running):
   - Visit http://localhost:3000/swagger-ui/ in your browser
   - This provides an interactive interface to explore and test the API endpoints

2. **Raw OpenAPI Specification**:
   - Visit http://localhost:3000/api-docs/openapi.json
   - This provides the raw OpenAPI specification in JSON format

### Documentation Access
To view the Rust API documentation:

1. Generate and serve the documentation:
```bash
# Option 1: Use the provided script
./serve-docs.sh

# Option 2: Manual commands
cargo doc
cd target/doc && python3 -m http.server 8000 --bind 0.0.0.0
```

2. Access the documentation in your browser:
- From your host machine: http://localhost:8000
- Navigate to the `fchain` crate documentation

## License

MIT 