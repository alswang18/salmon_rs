# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust project called `salmon_rs` using Rust 2024 edition. Currently a minimal project with a simple "Hello, world!" main function.

## Development Commands

### Build and Run
```bash
cargo build          # Build the project
cargo run            # Build and run the project
cargo check          # Quick compile check without building
```

### Testing
```bash
cargo test           # Run all tests
cargo test <name>    # Run specific test
```

### Code Quality
```bash
cargo fmt            # Format code
cargo clippy         # Run linter
```

## Project Structure

- `src/main.rs` - Entry point with main function
- `Cargo.toml` - Project configuration and dependencies (currently no external dependencies)
- `Cargo.lock` - Dependency lock file