## Development

### Setup

```bash
# Clone the repository
git clone https://github.com/yourusername/monocore.git
cd monocore/sdk/cpp

# Create a build directory
mkdir build && cd build

# Configure and build
cmake ..
make
```

### Running Tests

```bash
# In the build directory
ctest
```

### Publishing to Package Managers

#### Conan

Conan is our primary distribution method for C++ packages, offering flexibility and ease of publishing for early-stage projects.

1. Create a default Conan profile (if not already done):

```bash
# Create default profile based on your system
conan profile detect
```

2. Build and package using Conan:

```bash
# Create the package in your local cache
conan create .
```

3. Upload to your own repository or Conan Center (when ready):

```bash
# To your own repository (first add your remote)
conan remote add myrepo [your-repository-url]
conan upload microsandbox/0.1.0 --remote=myrepo

# Or to Conan Center (requires additional verification)
conan upload microsandbox/0.1.0 --remote=conancenter
```

4. Users can then install your package with:

```bash
conan install --requires=microsandbox/0.1.0
```

#### Publishing to Conan Center

Getting your package into Conan Center makes it easily accessible to all Conan users. Here's how to do it:

1. **Prepare your package**:

   - Ensure your package has quality documentation
   - Follow the [Conan Center guidelines](https://github.com/conan-io/conan-center-index/blob/master/docs/how_to_add_packages.md)
   - Have proper license files (e.g., LICENSE file in your repository)
   - Ensure your package builds and tests on multiple platforms

2. **Submit to Conan Center Index**:

   - Conan Center Index is a GitHub repository containing recipes for all packages on Conan Center
   - Fork the [Conan Center Index repository](https://github.com/conan-io/conan-center-index)
   - Create a new recipe for your package in the `recipes/microsandbox/all` directory
   - Submit a pull request to the Conan Center Index repository

3. **Recipe Requirements**:
   - Create a `conandata.yml` file with source code references
   - Create a `conanfile.py` that follows CCI standards
   - Examples and test package folder

Here's a basic example of what your Conan Center submission files might look like:

```yaml
# conandata.yml
sources:
  "0.1.0":
    url: "https://github.com/yourusername/monocore/archive/refs/tags/v0.1.0.tar.gz"
    sha256: "<SHA256 of your source archive>"
```

```python
# conanfile.py for Conan Center Index
from conan import ConanFile
from conan.tools.cmake import CMake, cmake_layout, CMakeToolchain
from conan.tools.files import get, copy

class MicrosandboxConan(ConanFile):
    name = "microsandbox"
    description = "Microsandbox C++ library for improved security through sandboxing"
    license = "MIT"
    url = "https://github.com/conan-io/conan-center-index"
    homepage = "https://github.com/yourusername/monocore"
    topics = ("sandbox", "security", "isolation")

    package_type = "library"
    settings = "os", "arch", "compiler", "build_type"
    options = {"shared": [True, False], "fPIC": [True, False]}
    default_options = {"shared": False, "fPIC": True}

    def config_options(self):
        if self.settings.os == "Windows":
            del self.options.fPIC

    def layout(self):
        cmake_layout(self)

    def source(self):
        get(self, **self.conan_data["sources"][self.version], strip_root=True)

    def generate(self):
        tc = CMakeToolchain(self)
        tc.generate()

    def build(self):
        cmake = CMake(self)
        cmake.configure()
        cmake.build()

    def package(self):
        copy(self, "LICENSE", src=self.source_folder, dst=os.path.join(self.package_folder, "licenses"))
        cmake = CMake(self)
        cmake.install()

    def package_info(self):
        self.cpp_info.libs = ["microsandbox"]
```

4. **Review Process**:

   - Conan Center maintainers will review your PR
   - CI will check your package builds on different platforms
   - You may need to address feedback and update your PR
   - Once approved, your package will be available on Conan Center

5. **Maintenance**:
   - You'll need to keep your recipe updated when new versions are released
   - Submit new PRs to update versions in Conan Center Index

For more comprehensive information about creating and maintaining Conan packages, see the [Conan 2.x documentation](https://docs.conan.io/2/) and [Conan Center Index documentation](https://github.com/conan-io/conan-center-index/tree/master/docs).

### Other Distribution Methods

As the project matures, consider:

1. **CMake FetchContent**: Enable direct inclusion in CMake projects
2. **GitHub releases**: Create tagged releases for direct download
3. **Header-only distribution**: If applicable, consider a header-only version for simpler integration
