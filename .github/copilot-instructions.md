# Copilot Instructions for git-repos

## Project Overview
`git-repos` is a Rust CLI tool for managing git repositories. It uses a TUI (Text User Interface) built with `ratatui` for an enhanced user experience. The project is designed to be lightweight with minimal dependencies, focusing on core functionality.

## Project Structure
- `src/main.rs` - Entry point for the CLI application
- `Cargo.toml` - Uses Rust 2024 edition with minimal dependencies currently

## Detailed Technical Development Process (DTDP)

When you add a new feature or fix a bug, follow these phases:
1. **Investigation Phase**: Understand the existing codebase and the context of the feature/bug.
2. **Discussion Phase**: Collaborate with the user to explore the problem/solution space BEFORE taking action.
3. **Action Phase**: Implement the agreed-upon solution after user approval.
4. **Summary Phase**: Write a brief summary of what was discussed, decided, and done.

Always test code quality before committing changes.

## Commit Message Guidelines
- Use imperative mood (e.g., "Add feature", "Fix bug")
- Use concise and descriptive messages
- Reference relevant issues or tickets when applicable
- Use conventional commits format:
  - `feat: ` for new features
  - `fix: ` for bug fixes
  - `docs: ` for documentation changes
  - `style: ` for code style changes (formatting, etc.)
  - `refactor: ` for code refactoring without changing functionality
  - `test: ` for adding or updating tests
  - `chore: ` for maintenance tasks (build scripts, dependencies, etc.)

## Development Workflows

### Building and Running
```powershell
# Build the project
cargo build

# Run the application
cargo run

# Run with arguments
cargo run -- <args>

# Build optimized release version
cargo build --release
```

### Testing
```powershell
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture
```

### Code Quality
```powershell
# Check code without building
cargo check

# Run clippy linter
cargo clippy

# Format code
cargo fmt
```

## Conventions

### Rust Edition
- Uses Rust 2024 edition - ensure code follows latest Rust idioms and syntax

### Documentation
- Always document public API
- Document non-trivial functions and modules with comments
- Keep the documentation up to date

### Architecture
- Follow modular design principles
- Separate concerns into different modules
- Keep functions small and focused on a single task

### Error Handling
- Prefer `Result<T, E>` for fallible operations
- Use `color_eyre` crate when adding error handling (to be added to dependencies)

### CLI Development
- Use mainly `ratatui` for terminal UI components when needed
- When adding CLI functionality, consider using `clap` for argument parsing
  - Structure commands as subcommands for extensibility

## License
MIT License - Copyright (c) Nicolas Arnaud-Cormos
