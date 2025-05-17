## Development

### Setup

```bash
# Clone the repository
git clone https://github.com/yourusername/monocore.git
cd monocore/sdk/dart

# Get dependencies
dart pub get
```

### Running Tests

```bash
dart test
```

### Publishing to pub.dev

1. Create an account on [pub.dev](https://pub.dev/) if you don't have one.

2. Ensure you have the required files:
   - `LICENSE` file in the root directory
   - `CHANGELOG.md` file documenting version changes
   - Updated SDK constraints in `pubspec.yaml`

3. Verify your package:

```bash
dart pub publish --dry-run
```

4. Publish your package:

```bash
dart pub publish
```

For more details, refer to [Publishing packages](https://dart.dev/tools/pub/publishing).
