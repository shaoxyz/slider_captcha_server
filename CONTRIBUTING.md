# Contributing to Slider Captcha Server

Thank you for your interest in contributing! 🎉

## 🚀 Quick Start

1. **Fork the repository**
2. **Clone your fork**
   ```bash
   git clone https://github.com/shaoxyz/slider_captcha_server
   cd slider_captcha_server
   ```
3. **Create a branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```
4. **Make your changes**
5. **Test your changes**
   ```bash
   cargo test
   cargo run --example actix_production --release
   cargo run --example benchmark --release
   ```
6. **Commit and push**
   ```bash
   git add .
   git commit -m "feat: add your feature description"
   git push origin feature/your-feature-name
   ```
7. **Create a Pull Request**

## 📝 Guidelines

### Code Style
- Follow standard Rust conventions
- Run `cargo fmt` before committing
- Run `cargo clippy` and fix warnings

### Commit Messages
Use conventional commits format:
- `feat:` - New feature
- `fix:` - Bug fix
- `perf:` - Performance improvement
- `docs:` - Documentation changes
- `test:` - Test additions/changes
- `refactor:` - Code refactoring

### Testing
- Add tests for new features
- Ensure all tests pass: `cargo test`
- Run benchmarks if performance-related: `cargo run --example benchmark --release`

### Documentation
- Update README if adding features
- Add code comments for complex logic
- Update CHANGELOG.md

## 🐛 Reporting Issues

When reporting bugs, please include:
- Rust version (`rustc --version`)
- Operating system
- Steps to reproduce
- Expected vs actual behavior
- Error messages or logs

## 💡 Feature Requests

We welcome feature requests! Please:
- Check if it's already requested
- Clearly describe the use case
- Explain why it would be valuable

## 📄 License

By contributing, you agree that your contributions will be licensed under GPL-3.0.

---

Thank you for making Slider Captcha Server better! 🙏

