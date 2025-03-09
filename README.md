# FChain - A Simple Blockchain in Rust

![Rust CI](https://github.com/YOUR_USERNAME/fchain/workflows/Rust%20CI/badge.svg)

> **Note:** After forking this repository, replace `YOUR_USERNAME` in the badge URL with your GitHub username.

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
- **Port 8000**: Documentation server (when running)

### API Access
When the application is running, you can access the API at:
- From inside the container: http://localhost:3000
- From your host machine: http://localhost:3000

### Documentation Access
To view the documentation:

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

## API Endpoints

- `GET /blocks` - Get all blocks in the chain
- `POST /transactions` - Create a new transaction
- `POST /mine` - Mine a new block with pending transactions
- `GET /chain/validate` - Validate the integrity of the blockchain

## License

MIT 