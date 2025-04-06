## Development

### Setup

```bash
# Clone the repository
git clone https://github.com/yourusername/monocore.git
cd monocore/sdk/crystal

# Install dependencies
shards install
```

### Running Tests

```bash
crystal spec
```

### Publishing

1. Update version number in:
   - `src/microsandbox/version.cr`
   - `shard.yml`
   - Update `CHANGELOG.md` with the new version

2. Build and test the package:
```bash
shards build
crystal spec
```

3. Tag the release:
```bash
git tag v0.1.0  # Replace with your version
git push origin v0.1.0
```

4. Publish to Crystal Shards:
   - Ensure your shard.yml is properly configured with:
     - name
     - version
     - description
     - repository URL
   - Push to GitHub
   - The package will be available via shards once the git tag is pushed

For more details, see the [Crystal Shards documentation](https://crystal-lang.org/reference/1.8/guides/hosting/shards.html).
