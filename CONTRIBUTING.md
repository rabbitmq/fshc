# Contributing to fshc

Thank you for your interest in contributing to fshc!

## Development Setup

1. Clone the repository:
   ```shell
   git clone https://github.com/rabbitmq/fshc.git
   cd fshc
   ```

2. Build the project:
   ```shell
   cargo build
   ```

## Running Tests

This project uses [cargo-nextest](https://nexte.st/) for running tests:

```shell
# Install nextest (if not already installed)
cargo install cargo-nextest

# Run all tests
cargo nextest run

# Run tests with retries for flaky tests
cargo nextest run --retries 2
```

You can also use the standard cargo test runner:

```shell
cargo test
```

## Code Quality

Before submitting a pull request, please ensure:

1. **Formatting**: Code is formatted with rustfmt
   ```shell
   cargo fmt --all
   ```

2. **Linting**: Code passes clippy checks
   ```shell
   cargo clippy --all-targets
   ```

3. **Tests**: All tests pass
   ```shell
   cargo nextest run
   ```

## Pull Request Process

1. Fork the repository
2. Create a topic branch for your changes
3. Make your changes and ensure all checks pass
4. Submit a pull request against the `main` branch

## License

By contributing to fshc, you agree that your contributions will be dual-licensed under the MIT and Apache 2.0 licenses.
