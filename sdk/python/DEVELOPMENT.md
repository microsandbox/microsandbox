## Development

### Setup

```bash
# Clone the repository
git clone https://github.com/yourusername/monocore.git
cd monocore/sdk/python

# Install dependencies using uv
uv venv
source .venv/bin/activate  # On Windows: .venv\Scripts\activate

# Install the package in development mode
uv pip install -e ".[dev]"
```

### Building the Package

To build your package with uv:

```bash
uv build
```

This will create distribution files in the `dist/` directory:

- A wheel file (`.whl`) - Built distribution
- A source distribution (`.tar.gz`)

### Publishing to PyPI

#### Creating API Tokens

1. For TestPyPI: Create an account on [TestPyPI](https://test.pypi.org/account/register/) and generate an API token in your account settings.
2. For PyPI: Create an account on [PyPI](https://pypi.org/account/register/) and generate an API token in your account settings.

Save these tokens securely as you'll only see them once.

#### Publishing to TestPyPI (Recommended first step)

```bash
# Build the distribution
uv build

# Upload to TestPyPI
uv publish --token $TEST_PYPI_TOKEN --publish-url https://test.pypi.org/legacy/
```

#### Verify your package can be installed from TestPyPI

```bash
# Create a new environment for testing
uv venv test_env
source test_env/bin/activate  # On Windows: test_env\Scripts\activate

# Install from TestPyPI
uv pip install --index-url https://test.pypi.org/simple/ --extra-index-url https://pypi.org/simple/ your-package-name
```

#### Publishing to PyPI

Once you've verified your package works correctly on TestPyPI:

```bash
# Build the distribution (if you haven't already)
uv build

# Upload to PyPI
uv publish --token $PYPI_TOKEN
```

### GitHub Actions for Automated Publishing

For automated releases, consider setting up a GitHub Actions workflow that publishes your package when you create a new release. For this approach, you'll need to:

1. Add your PyPI token as a GitHub repository secret
2. Create a workflow file (e.g., `.github/workflows/publish.yml`)

```yaml
name: Publish Python Package

on:
  release:
    types: [created]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Set up Python
        uses: actions/setup-python@v4
        with:
          python-version: "3.x"
      - name: Install dependencies
        run: |
          python -m pip install --upgrade pip
          pip install uv
      - name: Build and publish
        env:
          PYPI_TOKEN: ${{ secrets.PYPI_TOKEN }}
        run: |
          uv build
          uv publish --token $PYPI_TOKEN
```
