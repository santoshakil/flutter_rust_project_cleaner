# Flutter Rust Project Cleaner (frpc)

A high-performance, enterprise-grade CLI tool for cleaning Flutter and Rust projects. Recursively finds and cleans build artifacts to free up disk space.

## Features

- 🚀 **Fast**: Parallel scanning and cleaning with configurable concurrency
- 🎯 **Smart**: Automatically detects Flutter, Rust, and mixed projects
- 🛡️ **Safe**: Dry-run mode, interactive selection, and confirmation prompts
- 📊 **Informative**: Progress bars, colored output, and space usage estimates
- ⚙️ **Configurable**: Customizable exclusion patterns and command arguments
- 🔄 **Flexible**: Multiple operating modes (clean, list, config)
- 🎛️ **Advanced Error Handling**: Detailed error reporting with specific error types
- 📄 **JSON Output**: Machine-readable output for automation
- 🚦 **Graceful Interruption**: Ctrl+C handling for clean shutdown
- 🔐 **Permission Aware**: Checks permissions before attempting operations

## Installation

### From Source

```bash
cargo install --path .
```

### Pre-built Binaries

Download the latest release from the [releases page](https://github.com/yourname/flutter_rust_project_cleaner/releases).

## Usage

### Basic Commands

```bash
# Clean all projects in current directory
frpc clean .

# Clean with dry-run to see what would be cleaned
frpc clean . --dry-run

# Clean only Flutter projects
frpc clean . -t flutter

# Clean with interactive selection
frpc clean . --interactive

# List all projects without cleaning
frpc list .

# Generate shell completions
frpc completions bash > /etc/bash_completion.d/frpc
```

### Advanced Options

```bash
# Clean with custom parallelism
frpc clean . -j 4

# Exclude specific directories
frpc clean . --exclude "vendor/*" --exclude "third_party/*"

# Set maximum search depth
frpc clean . --max-depth 3

# Clean specific project types
frpc clean . -t flutter -t rust

# Verbose output
frpc clean . -vv

# JSON output for automation
frpc clean . --json

# Quiet mode (no progress bars)
frpc clean . --quiet
```

### Configuration

```bash
# Initialize configuration
frpc config init

# Show current configuration
frpc config show

# Edit configuration
frpc config edit
```

## Configuration File

The configuration file is located at `~/.config/frpc/config.toml`:

```toml
# Default exclusion patterns
default_exclude = ["node_modules", ".git", "target", "build"]

# Flutter clean command arguments
flutter_clean_args = ["clean"]

# Cargo clean command arguments
cargo_clean_args = ["clean"]

# Maximum parallel jobs (null for auto-detect)
max_parallel_jobs = null

# Interactive mode by default
interactive_by_default = false

# Show progress bars
show_progress = true

# Confirm before cleaning
confirm_before_clean = true
```

## Project Types

- **Flutter**: Projects with `pubspec.yaml`
- **Rust**: Projects with `Cargo.toml`
- **Mixed**: Projects with both `pubspec.yaml` and `Cargo.toml`

## What Gets Cleaned

### Flutter Projects
- `.dart_tool/`
- `build/`
- `.flutter-plugins-dependencies`

### Rust Projects
- `target/`

## Safety Features

1. **Dry Run**: Preview what would be cleaned without actually doing it
2. **Interactive Mode**: Select specific projects to clean
3. **Confirmation Prompt**: Confirm before cleaning (configurable)
4. **Progress Tracking**: Real-time feedback on cleaning progress
5. **Error Handling**: Graceful handling of permission errors and missing tools

## Performance

- Parallel directory scanning using Rayon
- Concurrent project cleaning with configurable parallelism
- Efficient space calculation with caching
- Minimal memory footprint

## License

MIT License - see LICENSE file for details

## Contributing

Contributions are welcome! Please see CONTRIBUTING.md for guidelines.

## Changelog

See CHANGELOG.md for version history.