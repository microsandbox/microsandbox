## Development

### Setup

```bash
# Clone the repository
git clone https://github.com/yourusername/monocore.git
cd monocore/sdk/julia

# Start Julia
julia --project=.

# Press ']' to enter Pkg mode
# Activate and install dependencies
activate .
instantiate
```

### Running Tests

```julia
# In Pkg mode (press ']' from the Julia REPL)
test Microsandbox
```

### Building Documentation

```julia
using Pkg
Pkg.add("Documenter")
using Documenter, Microsandbox
makedocs(modules=[Microsandbox])
```

### Publishing to Julia Registry

1. Create a GitHub repository for your package.

2. Tag a release:

```bash
git tag v0.1.0
git push origin v0.1.0
```

3. Register the package with the Julia Registry:

There are two recommended ways to register your package:

**Option 1: Using the Registrator GitHub App (Recommended)**

Install the [JuliaRegistrator GitHub App](https://github.com/apps/juliateam-registrator) and comment `@JuliaRegistrator register` on the commit or release you want to register.

**Option 2: Manual registration**

Fork the [General Registry](https://github.com/JuliaRegistries/General), add your package information following the registry's format, and submit a pull request.
