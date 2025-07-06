# Contributing to AdBlock

Thank you for your interest in contributing to AdBlock! We welcome contributions from everyone.

## 🤝 How to Contribute

### Reporting Bugs

1. Check if the bug has already been reported in [Issues](https://github.com/ayutaz/block-ad/issues)
2. Create a new issue with the bug report template
3. Include:
   - Clear description of the bug
   - Steps to reproduce
   - Expected behavior
   - Actual behavior
   - Device/OS information
   - Screenshots if applicable

### Suggesting Features

1. Check existing [Issues](https://github.com/ayutaz/block-ad/issues) and [Discussions](https://github.com/ayutaz/block-ad/discussions)
2. Create a new discussion or issue
3. Describe the feature and its benefits
4. Consider implementation details

### Code Contributions

1. **Fork the repository**
2. **Create a feature branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```
3. **Follow coding standards**
   - Rust: Run `cargo fmt` and `cargo clippy`
   - Kotlin: Follow Android Studio formatting
   - Swift: Follow Xcode formatting
4. **Write tests**
   - Add unit tests for new functionality
   - Ensure all tests pass
5. **Commit your changes**
   ```bash
   git commit -m "feat: add new feature"
   ```
   Follow [Conventional Commits](https://www.conventionalcommits.org/)
6. **Push to your fork**
   ```bash
   git push origin feature/your-feature-name
   ```
7. **Create a Pull Request**

## 📋 Pull Request Guidelines

- Fill out the PR template completely
- Link related issues
- Ensure CI/CD passes
- Request review from maintainers
- Respond to feedback promptly

## 🧪 Testing

Before submitting:

```bash
# Run all tests
cargo test
cd android && ./gradlew test
cd ios && xcodebuild test -scheme AdBlock
```

## 💻 Development Setup

1. Install prerequisites:
   - Rust 1.70+
   - Android Studio
   - Xcode (macOS only)
   - cargo-ndk

2. Clone and setup:
   ```bash
   git clone https://github.com/ayutaz/block-ad.git
   cd block-ad
   ```

## 📝 Commit Message Format

We use [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation changes
- `style:` Code style changes
- `refactor:` Code refactoring
- `test:` Test additions/changes
- `chore:` Build process/auxiliary tool changes

## 🌍 Translations

Help translate AdBlock:

1. Check existing translations in `assets/locales/`
2. Add new language file
3. Submit PR with translation

## 📜 Code of Conduct

- Be respectful and inclusive
- Welcome newcomers
- Focus on constructive criticism
- No harassment or discrimination

## 🙏 Thank You!

Every contribution, no matter how small, is valuable and appreciated!