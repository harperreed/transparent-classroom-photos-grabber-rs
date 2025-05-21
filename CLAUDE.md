# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview
- Rust-based CRM application with SQLite database and vCard integration
- TUI and web interfaces for managing contacts, facts, and events

## Build & Test Commands
- `cargo build` - Build the project
- `cargo run` - Run the application
- `cargo test` - Run all tests
- `cargo test <test_name>` - Run a specific test
- `cargo clippy` - Run the linter

## Code Style Guidelines
- Follow Rust idioms and naming conventions
- Use strong typing with proper error handling
- Prefer Result/Option over unwrap/expect in production code
- Document public APIs with rustdoc comments
- Organize imports alphabetically
- Format code with `rustfmt`
- Use TDD: write tests before implementation code
- Keep functions focused and small (< 50 lines when possible)
- Maintain consistent error handling patterns
