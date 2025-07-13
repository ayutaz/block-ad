# Contributing to AdBlock

Thank you for your interest in contributing to AdBlock! This document provides comprehensive guidelines for contributing to our open-source ad blocking solution.

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
   - Rust: Run `cargo fmt` and `cargo clippy -- -D warnings`
   - Kotlin: Use Android Studio's formatter (Ctrl+Alt+L / Cmd+Option+L)
   - Swift: Run SwiftLint (`swiftlint` in ios directory)
   - Python: Run `black` and `flake8` for E2E tests
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

### Running Tests

```bash
# Core library tests
cd core
cargo test
cargo test --release  # Also test in release mode

# Android tests
cd android
./gradlew test
./gradlew lint
./gradlew connectedAndroidTest  # Requires device/emulator

# iOS tests  
cd ios
xcodebuild test -scheme AdBlock -destination 'platform=iOS Simulator,name=iPhone 14'

# E2E tests
cd e2e_tests
python -m pytest
python test_youtube_blocking.py  # Specific test
```

### Test Coverage
- Aim for >80% code coverage
- All new features must include tests
- Bug fixes should include regression tests

### Writing Tests

**Rust Example:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_block_ad_domain() {
        let engine = AdBlockEngine::new();
        engine.load_rules("||doubleclick.net^");
        assert!(engine.should_block("https://doubleclick.net/ads"));
    }
}
```

**Kotlin Example:**
```kotlin
@Test
fun testVpnServiceStartup() {
    val service = AdBlockVpnService()
    assertTrue(service.startTunnel())
    assertTrue(service.isRunning)
}
```

## 💻 Development Setup

### Prerequisites

- **Rust**: 1.70.0 or higher
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android
  rustup target add aarch64-apple-ios x86_64-apple-ios
  ```

- **Android Development**:
  - Android Studio (latest stable)
  - Android NDK (installed via SDK Manager)
  - cargo-ndk: `cargo install cargo-ndk`
  - JDK 17 or higher

- **iOS Development** (macOS only):
  - Xcode 14.0 or higher
  - CocoaPods: `sudo gem install cocoapods`

- **Testing Tools**:
  - Python 3.8+: For E2E tests
  - Node.js 16+: For some build tools

### Clone and Setup

```bash
git clone https://github.com/ayutaz/block-ad.git
cd block-ad

# Setup Git hooks (optional but recommended)
cp scripts/pre-commit .git/hooks/
chmod +x .git/hooks/pre-commit
```

## 📝 Commit Message Format

We use [Conventional Commits](https://www.conventionalcommits.org/):

### Format
```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types
- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation changes
- `style:` Code style changes (formatting, etc.)
- `refactor:` Code refactoring
- `test:` Test additions/changes
- `chore:` Build process/auxiliary tool changes
- `perf:` Performance improvements
- `ci:` CI/CD changes

### Scopes
- `core`: Rust core library
- `android`: Android app
- `ios`: iOS app
- `e2e`: End-to-end tests
- `docs`: Documentation

### Examples
```
feat(android): add custom filter rule support

- Add UI for managing custom rules
- Implement rule persistence with SharedPreferences
- Add validation for rule syntax

Closes #123
```

```
fix(core): resolve memory leak in filter engine

The filter engine was not properly releasing compiled patterns.
This fix ensures all patterns are freed when the engine is destroyed.

Fixes #456
```

## 🌍 Translations

Help translate AdBlock:

1. Check existing translations in `assets/locales/`
2. Add new language file
3. Submit PR with translation

## 🔧 Coding Standards

### Rust (Core Library)
```rust
// Use clear, descriptive names
pub fn should_block_url(url: &str) -> bool { }

// Document public APIs
/// Checks if the given URL should be blocked based on filter rules
/// 
/// # Arguments
/// * `url` - The URL to check
/// 
/// # Returns
/// `true` if the URL matches any blocking rule
pub fn should_block_url(url: &str) -> bool {
    // Implementation
}

// Always run before committing
cargo fmt
cargo clippy -- -D warnings
```

### Kotlin (Android)
```kotlin
// Use descriptive names and follow Kotlin conventions
class AdBlockVpnService : VpnService() {
    /**
     * Starts the VPN tunnel for ad blocking
     * @return true if started successfully
     */
    fun startTunnel(): Boolean {
        // Implementation
    }
}

// Use Android Studio's formatter (Ctrl+Alt+L / Cmd+Option+L)
```

### Swift (iOS)
```swift
// Follow Swift API Design Guidelines
public class AdBlockEngine {
    /// Loads filter rules from the specified list
    /// - Parameter filterList: The filter list content
    /// - Returns: Success status
    public func loadFilterList(_ filterList: String) -> Bool {
        // Implementation
    }
}

// Use SwiftLint for consistent style
```

## 📜 Code of Conduct

### Our Pledge
We pledge to make participation in our project a harassment-free experience for everyone, regardless of age, body size, disability, ethnicity, gender identity, level of experience, nationality, personal appearance, race, religion, or sexual identity and orientation.

### Expected Behavior
- Be respectful and inclusive
- Welcome newcomers and help them get started
- Focus on constructive criticism
- Accept feedback gracefully
- Show empathy towards other community members

### Unacceptable Behavior
- Harassment, discrimination, or hate speech
- Personal attacks or trolling
- Publishing others' private information
- Other conduct that could be considered inappropriate

### Enforcement
Instances of abusive, harassing, or otherwise unacceptable behavior may be reported to the project maintainers. All complaints will be reviewed and investigated promptly and fairly.

## 🙏 Thank You!

Every contribution, no matter how small, is valuable and appreciated!