# Contributing to Settlement Program

Thank you for your interest in contributing to the 1024EX Settlement Program!

## 🚀 Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/1024-settlement-program.git`
3. Create a branch: `git checkout -b feature/your-feature-name`
4. Make your changes
5. Run tests: `cargo test --lib`
6. Commit: `git commit -m "Add your feature"`
7. Push: `git push origin feature/your-feature-name`
8. Open a Pull Request

## 📋 Development Guidelines

### Code Style

- Follow Rust standard formatting: `cargo fmt`
- Pass clippy lints: `cargo clippy`
- Write tests for new features
- Document public APIs

### Commit Messages

Use clear, descriptive commit messages:

```
feat: Add new validation for trade data
fix: Correct hash calculation in utils
docs: Update API documentation
test: Add tests for edge cases
```

### Testing

All code changes should include tests:

```bash
# Run all tests
cargo test --lib

# Run specific test
cargo test test_name

# All tests must pass before PR
```

### Documentation

- Update README.md if adding features
- Add inline comments for complex logic
- Update API documentation in docs/

## 🐛 Reporting Bugs

Use GitHub Issues with the following information:

- Clear description of the bug
- Steps to reproduce
- Expected vs actual behavior
- Environment (Rust version, OS, etc.)
- Relevant logs or error messages

## 💡 Suggesting Features

Open a GitHub Issue with:

- Clear feature description
- Use case and motivation
- Proposed implementation (if any)

## 📝 Pull Request Process

1. Ensure tests pass
2. Update documentation
3. Link related issues
4. Wait for review
5. Address feedback

## 🤝 Code of Conduct

- Be respectful and inclusive
- Focus on constructive feedback
- Help others learn and grow

## 📞 Questions?

- Open a GitHub Discussion
- Check existing Issues and PRs
- Read the documentation in docs/

Thank you for contributing! 🎉

