# Simple Devcontainer Setup ✨

I've created a minimal devcontainer configuration for your Rust proc-macro workshop project focused on simplicity and learning.

## What's Included

### Rust Environment
- ✅ Official latest Rust Docker image
- ✅ rustfmt for code formatting
- ✅ rust-src for IDE support
- ✅ Root access for installing additional tools as needed

### VS Code Integration
- ✅ Rust Analyzer extension only
- ✅ Proc-macro support enabled
- ✅ Format on save with rustfmt
- ✅ Simple debugging setup

### Development Features
- ✅ IntelliSense and code completion
- ✅ Proc-macro expansion support
- ✅ Basic debugging for tests and binaries
- ✅ Simple build/test/format tasks

## Files Created

- **`.devcontainer/devcontainer.json`** - Minimal devcontainer config
- **`.devcontainer/Dockerfile`** - Simple Rust image setup
- **`.vscode/settings.json`** - Basic Rust settings
- **`.vscode/tasks.json`** - Build, test, and format tasks
- **`.vscode/launch.json`** - Simple debugging setup

## How to Use

1. **Open in VS Code** with the Dev Containers extension
2. **Reopen in Container** when prompted
3. **Start coding!** Everything is ready

## Available Commands

### Tasks (Ctrl/Cmd+Shift+P → "Tasks: Run Task")
- **Build** - Build the project
- **Test** - Run all tests
- **Format** - Format all Rust code

### Debugging (F5 or Debug panel)
- **Debug unit tests** - Debug any unit tests
- **Debug workshop binary** - Debug the main workshop executable

### Terminal Commands
```bash
# Build everything
cargo build

# Run all tests
cargo test

# Run tests for specific project
cargo test -p bitfield

# Format code
cargo fmt

# Install additional tools (you have root access!)
cargo install cargo-expand
cargo expand  # See generated macro code
```

## Root Access for Learning

The container runs as root, so you can:
- Install any additional cargo tools you need
- Install system packages with `apt install`
- Experiment and modify the environment freely

This setup is perfect for learning Rust procedural macros! 🦀
