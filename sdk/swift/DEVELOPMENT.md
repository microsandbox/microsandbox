# Development Guide

This document contains instructions for developing the Microsandbox Swift SDK.

## Development

### Setup

```bash
# Clone the repository
git clone https://github.com/yourusername/monocore.git
cd monocore/sdk/swift

# Build the package
swift build
```

### Running Tests

```bash
swift test
```

### Publishing with Swift Package Manager

Swift Package Manager uses Git tags to identify package versions. In a monorepo context, there are a few approaches:

#### Option 1: Prefixed Tags (Recommended for Monorepos)

For a monorepo where this Swift package is just one component:

```bash
git tag sdk/swift/0.1.0
git push origin sdk/swift/0.1.0
```

Users would then reference your package with:

```swift
.package(url: "https://github.com/yourusername/monorepo.git", .exact("sdk/swift/0.1.0"))
```

#### Option 2: Separate Repository

If you prefer, you can extract the Swift SDK into its own repository for cleaner versioning:

```bash
git tag v0.1.0
git push origin v0.1.0
```

#### Option 3: Using Exact Commits

For tight integration within a monorepo workflow, you can reference specific commits:

```swift
.package(url: "https://github.com/yourusername/monorepo.git", .revision("commit-hash"))
```

For more details, refer to [Swift Package Manager documentation](https://swift.org/package-manager/).

### CocoaPods

To publish a new version to CocoaPods from a monorepo:

1. Update the version in `Microsandbox.podspec` to `0.1.0`
2. Make sure all changes are committed and pushed
3. Tag the release (using a prefixed tag for monorepos):
   ```bash
   git tag -a sdk/swift/0.1.0 -m "Release Swift SDK 0.1.0"
   git push origin sdk/swift/0.1.0
   ```
4. Update the podspec's source to reference the tag:
   ```ruby
   s.source = { :git => 'https://github.com/microsandbox/microsandbox.git', :tag => 'sdk/swift/0.1.0' }
   ```
5. Publish to CocoaPods:
   ```bash
   pod trunk push sdk/swift/Microsandbox.podspec
   ```

Note: You need to be registered with CocoaPods trunk to publish. If you haven't registered yet, run:

```bash
pod trunk register your.email@example.com 'Your Name'
```
