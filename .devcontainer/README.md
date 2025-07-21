# Development Container

This directory contains the development container configuration for the Rust Proc-Macro Workshop.

## What's Included

The devcontainer provides a complete Rust development environment with:

### Rust Toolchain
- Latest stable Rust compiler
- Clippy (linter)
- Rustfmt (formatter)
- Rust-src (for IDE support)
- Rust-analyzer (language server)

### Development Tools
- **cargo-expand** - Expand macros and see generated code
- **cargo-watch** - Watch for changes and run commands automatically
- **cargo-edit** - Add/remove dependencies from command line
- **cargo-audit** - Security audit for dependencies
- **cargo-outdated** - Check for outdated dependencies

### VS Code Extensions
- **Rust Analyzer** - Rich language support for Rust
- **Even Better TOML** - TOML file support
- **Crates** - Manage Rust dependencies
- **CodeLLDB** - Debugger support
- **Error Lens** - Inline error display
- **GitLens** - Enhanced Git capabilities

### Debugging Tools
- GDB and LLDB debuggers
- Built-in VS Code debugging support

## Getting Started

1. Make sure you have VS Code with the Dev Containers extension installed
2. Open the project folder in VS Code
3. When prompted, click "Reopen in Container" or use the command palette:
   - `Ctrl/Cmd + Shift + P`
   - Type "Dev Containers: Reopen in Container"
4. Wait for the container to build and start
5. The environment will be ready for Rust proc-macro development!

## Useful Commands

Once inside the container, you can use these commands:

```bash
# Build the entire workspace
cargo build

# Run tests for all projects
cargo test

# Run tests for a specific project (e.g., builder)
cargo test -p derive_builder

# Expand macros to see generated code
cargo expand

# Watch for changes and run tests
cargo watch -x test

# Format code
cargo fmt

# Run clippy linter
cargo clippy

# Check for security vulnerabilities
cargo audit

# Check for outdated dependencies
cargo outdated
```

## Working with Proc-Macros

The development environment is specifically configured for proc-macro development:

- **Rust Analyzer** is configured to enable proc-macro expansion
- **cargo-expand** is available to see the generated code from your macros
- Error highlighting and IntelliSense work with proc-macro code
- Debugging support is available for stepping through macro code

## Performance

The devcontainer is configured with:
- Cargo registry and git cache mounting for faster dependency downloads
- Incremental compilation enabled
- Rust-analyzer optimized for proc-macro projects

## Customization

You can customize the environment by modifying:
- `.devcontainer/devcontainer.json` - VS Code settings and extensions
- `.devcontainer/Dockerfile` - Additional tools and system packages
