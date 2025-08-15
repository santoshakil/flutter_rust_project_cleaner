# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview
Enterprise-grade CLI tool for cleaning Flutter and Rust projects. Features parallel processing, progress tracking, interactive mode, and extensive configuration options.

## Architecture

### Module Structure
- `src/main.rs` - Entry point and command orchestration
- `src/config/` - CLI parsing and configuration management
- `src/error.rs` - Custom error types with thiserror
- `src/project/` - Project detection and metadata collection
- `src/scanner/` - Parallel directory scanning
- `src/cleaner/` - Concurrent project cleaning
- `src/utils/` - Logging and interactive utilities

### Key Design Patterns
- Builder pattern for Scanner and Cleaner configuration
- Parallel processing with Rayon
- Async/await for concurrent operations
- Type-safe error handling with custom error types
- Progress reporting with indicatif

## Common Commands

```bash
# Build and run
cargo build --release
cargo run -- clean . --dry-run

# Testing
cargo test
cargo test --all-features

# Code quality
cargo fmt
cargo clippy -- -D warnings
cargo doc --no-deps

# Benchmarks
cargo bench

# Release build
cargo build --release --target x86_64-unknown-linux-musl
```

## Development Workflow

1. Make changes following existing patterns
2. Run `cargo check` for quick compilation check
3. Run `cargo clippy` for lint issues
4. Run `cargo fmt` for formatting
5. Run `cargo test` for unit/integration tests
6. Update documentation if needed

## Important Patterns

### Error Handling
- Use custom `CleanerError` type
- Convert errors with `?` operator
- Add context with error messages

### Progress Reporting
- Use indicatif for progress bars
- MultiProgress for concurrent operations
- Spinner style for unknown duration tasks

### Configuration
- Clap for CLI parsing with derive API
- TOML configuration file support
- Environment variable support via clap

### Testing
- Unit tests in module files
- Integration tests in tests/
- Use tempfile for filesystem tests
- Mock external commands when possible

## CI/CD
- GitHub Actions for testing across platforms
- Automated releases with multiple targets
- Code coverage with tarpaulin
- Cross-compilation for Linux musl