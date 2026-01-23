# Contributing

We love contributions! Here is how to get started.

## Development

This is a Rust project managed by Cargo.

### Build

Build the project:

```bash
cargo build
```

### Test

Run the test suite:

```bash
cargo test
```

Ensure code is linted and formatted:

```bash
cargo clippy
cargo fmt
```

## Pull Requests

- **Use Conventional Commits**: We follow [Conventional Commits](https://www.conventionalcommits.org/). Start your commit messages with `feat:`, `fix:`, `docs:`, etc. This is required for our changelog generation (via `git cliff`).
- **Keep it small**: Focus on one logical change per pull request.
- **Test your changes**: Run the tests above before submitting.

Thank you for helping improve Recoco!
