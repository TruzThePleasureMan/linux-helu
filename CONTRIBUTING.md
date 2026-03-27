# Contributing to Linux Helu

Welcome! We are excited that you are interested in contributing to Linux Helu. Note that this project started as a meme, so contributions of all sizes are welcome!

## Setting up the Dev Environment

To set up your local development environment, you will need the following dependencies:
- Rust toolchain (`cargo`)
- `libwebkit2gtk-4.1-dev` and common build essentials.
- Tauri CLI (`npm install -g @tauri-apps/cli`)
- ONNX Runtime and a valid `mobilefacenet.onnx` model (InsightFace). Download via `./scripts/download-model.sh`
- PostgreSQL (if you are working on `helu-server`)

## Running with Mock Hardware

To run the daemon and UI with mocked hardware, run them with the `--mock` flag. Ensure your `helu.toml` is set to `bus = "session"`, and then execute:
```bash
cargo run --bin helud
```
And in a separate terminal:
```bash
cargo run --bin helu-ui -- --mock
```

## Running Tests

To run the test suite across the workspace, use:
```bash
cargo test --workspace
```

## PR Expectations

Before submitting a Pull Request, please ensure the following:
1. `cargo clippy --workspace -- -D warnings` must pass without any warnings.
2. `cargo test --workspace` must pass successfully.

## Issue Labels

- `bug`: Something isn't working as expected.
- `enhancement`: New feature or request.
- `documentation`: Improvements or additions to documentation.
- `help wanted`: Extra attention is needed.

## Building on top of helud

If you want to build your own tools on top of `helud`, please see the [D-Bus interface spec in ARCHITECTURE.md](ARCHITECTURE.md#d-bus-interface-spec) for details.
