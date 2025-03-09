# Contributing to FChain

Thank you for considering contributing to FChain! This document provides guidelines and instructions for contributing to this project.

## Code of Conduct

Please be respectful and considerate of others when contributing to this project.

## How to Contribute

1. Fork the repository
2. Create a new branch for your feature or bugfix
3. Make your changes
4. Run tests to ensure your changes don't break existing functionality
5. Submit a pull request

## Development Environment

This project uses Dev Containers for a consistent development environment. To get started:

1. Install [Docker](https://www.docker.com/products/docker-desktop) and [VS Code](https://code.visualstudio.com/)
2. Install the [Remote - Containers](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers) extension for VS Code
3. Clone this repository
4. Open the project in VS Code and click "Reopen in Container" when prompted

## Testing

Before submitting a pull request, make sure all tests pass:

```bash
cargo test
```

## Code Style

This project follows the Rust standard code style. Please ensure your code is formatted with:

```bash
cargo fmt
```

And passes linting with:

```bash
cargo clippy
```

## Continuous Integration

This project uses GitHub Actions for continuous integration. The CI pipeline will automatically run on your pull request to check:

- Code formatting (rustfmt)
- Linting (clippy)
- Building the project
- Running all tests

## Pull Request Process

1. Update the README.md with details of changes if applicable
2. Update the documentation if you're changing functionality
3. The PR should work with the CI pipeline passing all checks
4. Your PR will be reviewed by a maintainer

## License

By contributing to this project, you agree that your contributions will be licensed under the project's MIT license. 