# Contributing to Conduit

Thank you for your interest in contributing to Conduit! This document provides guidelines and information for contributors.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [How to Contribute](#how-to-contribute)
- [Pull Request Process](#pull-request-process)
- [Code Style](#code-style)
- [Testing](#testing)
- [Reporting Bugs](#reporting-bugs)
- [Suggesting Features](#suggesting-features)

## Code of Conduct

This project adheres to the [Contributor Covenant Code of Conduct](https://www.contributor-covenant.org/version/2/1/code_of_conduct/). By participating, you are expected to uphold this code. Please report unacceptable behavior to the maintainers.

## Getting Started

1. Fork the repository on GitHub
2. Clone your fork locally
3. Set up the development environment (see below)
4. Create a branch for your changes
5. Make your changes and test them
6. Submit a pull request

## Development Setup

### Prerequisites

- **Rust 1.70+** - Install via [rustup](https://rustup.rs/)
- **At least one supported agent:**
  - [Claude Code](https://github.com/anthropics/claude-code) (`claude` binary)
  - [Codex CLI](https://github.com/openai/codex) (`codex` binary)

### Building from Source

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/conduit.git
cd conduit

# Build in debug mode
cargo build

# Build in release mode
cargo build --release

# Run tests
cargo test

# Run the application
cargo run
```

### Website Development

The website is built with [Astro](https://astro.build/):

```bash
cd website
npm install
npm run dev    # Start dev server at http://localhost:4321
npm run build  # Build for production
```

## How to Contribute

### Types of Contributions

- **Bug fixes** - Fix issues reported in the issue tracker
- **Features** - Implement new functionality
- **Documentation** - Improve docs, README, or inline comments
- **Tests** - Add or improve test coverage
- **Refactoring** - Code improvements without changing functionality

### Before You Start

1. Check existing [issues](https://github.com/conduit-cli/conduit/issues) to see if your idea has been discussed
2. For significant changes, open an issue first to discuss your approach
3. For small fixes, you can submit a PR directly

## Pull Request Process

1. **Create a descriptive branch name:**
   ```bash
   git checkout -b feature/your-feature-name
   git checkout -b fix/issue-description
   ```

2. **Make your changes:**
   - Write clear, concise commit messages
   - Keep commits focused and atomic
   - Reference issue numbers in commits when applicable

3. **Test your changes:**
   ```bash
   cargo test
   cargo clippy -- -D warnings
   cargo fmt --check
   ```

4. **Update documentation** if your changes affect user-facing behavior

5. **Submit the PR:**
   - Fill out the PR template completely
   - Link any related issues
   - Request review from maintainers

6. **Respond to feedback:**
   - Address review comments promptly
   - Push additional commits as needed
   - Maintainers may request changes before merging

## Code Style

### Rust

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` to format code
- Use `cargo clippy` to catch common mistakes
- Write descriptive variable and function names
- Add comments for complex logic

### TypeScript/JavaScript (Website)

- Use TypeScript where possible
- Follow existing patterns in the codebase
- Use meaningful component and function names

### General

- Keep functions focused and small
- Prefer explicit over implicit behavior
- Write self-documenting code when possible
- Add comments for "why", not "what"

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run a specific test
cargo test test_name

# Run tests for a specific module
cargo test module_name::
```

### Writing Tests

- Add unit tests for new functions
- Add integration tests for new features
- Test edge cases and error conditions
- Use descriptive test names that explain what is being tested

## Reporting Bugs

When reporting a bug, please include:

1. **Environment information:**
   - Operating system and version
   - Rust version (`rustc --version`)
   - Conduit version (`conduit --version`)
   - Terminal emulator

2. **Steps to reproduce:**
   - Minimal steps to trigger the bug
   - Expected behavior
   - Actual behavior

3. **Additional context:**
   - Error messages or logs
   - Screenshots if applicable
   - Relevant configuration

Use the [bug report template](../../.github/ISSUE_TEMPLATE/bug_report.md) when creating issues.

## Suggesting Features

When suggesting a feature:

1. **Check existing issues** to avoid duplicates
2. **Describe the problem** your feature would solve
3. **Explain your proposed solution**
4. **Consider alternatives** you've thought about
5. **Provide examples** of how the feature would work

Use the [feature request template](../../.github/ISSUE_TEMPLATE/feature_request.md) when creating issues.

## Questions?

- Open a [discussion](https://github.com/conduit-cli/conduit/discussions) for questions
- Join the [Discord community](https://discord.gg/F9pfRd642H)
- Check the [documentation](https://getconduit.sh/docs)

Thank you for contributing to Conduit!
