# Contributing to Bold

Thank you for your interest in contributing to Bold! This document provides guidelines and information to help you contribute effectively.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Development Setup](#development-setup)
3. [Project Structure](#project-structure)
4. [Coding Standards](#coding-standards)
5. [Testing](#testing)
6. [Submitting Changes](#submitting-changes)
7. [Reporting Issues](#reporting-issues)

## Getting Started

Before you start contributing, please make sure you have:

1. Read the README.md file to understand the project
2. Installed Rust and Cargo (preferably using rustup)
3. Familiarized yourself with the NFS v4.0 protocol (RFC 7530)

## Development Setup

1. Fork the repository
2. Clone your fork:
   ```
   git clone https://github.com/your-username/bold.git
   cd bold
   ```
3. Install dependencies:
   ```
   cargo build
   ```

## Project Structure

The project is organized into several main components:

- `lib/` - Core library implementing the NFS server
- `exec/` - Executable binaries (bold-mem, bold-real-fs)
- `proto/` - Protocol definitions and serialization
- `tests/` - Integration and unit tests
- `examples/` - Example usage of the library
- `docs/` - Documentation files

Key modules in the library:

- `server/` - Main server implementation
- `server/filemanager/` - File management (both in-memory and real file system)
- `server/clientmanager/` - Client management
- `server/nfs40/` - NFS v4.0 protocol implementation

## Coding Standards

We follow standard Rust coding conventions:

1. Use `rustfmt` to format your code:
   ```
   cargo fmt
   ```
2. Use `clippy` to catch common mistakes:
   ```
   cargo clippy
   ```
3. Write clear, descriptive commit messages
4. Follow the existing code style in the project
5. Add documentation to public functions and structs
6. Write unit tests for new functionality

## Testing

We have both unit tests and integration tests:

1. Run unit tests:
   ```
   cargo test
   ```
2. Run integration tests:
   ```
   cargo test --test '*'
   ```

Before submitting changes, make sure all tests pass.

## Submitting Changes

1. Create a new branch for your changes:
   ```
   git checkout -b feature/your-feature-name
   ```
2. Make your changes and commit them with descriptive messages
3. Push your branch to your fork:
   ```
   git push origin feature/your-feature-name
   ```
4. Create a pull request against the main repository

Please include:

- A clear description of the changes
- Any relevant issue numbers
- Tests for new functionality
- Updated documentation if needed

## Reporting Issues

If you find a bug or have a feature request:

1. Check if there's already an open issue
2. If not, create a new issue with:
   - A clear title
   - Detailed description of the problem or feature
   - Steps to reproduce (for bugs)
   - Expected and actual behavior (for bugs)
   - Environment information (OS, Rust version, etc.)

## Code of Conduct

Please note that this project is released with a Contributor Code of Conduct. By participating in this project you agree to abide by its terms.

## Questions?

If you have any questions about contributing, feel free to open an issue or contact the maintainers.