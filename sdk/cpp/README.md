# Microsandbox C++ SDK

A minimal C++ SDK for the Microsandbox project.

## Installation

### Using CMake (recommended)

1. Clone this repository:

```bash
git clone https://github.com/yourusername/monocore.git
```

2. Add the SDK to your CMake project:

```cmake
# In your CMakeLists.txt
add_subdirectory(/path/to/monocore/sdk/cpp)
target_link_libraries(your_target microsandbox)
```

### Using vcpkg

```bash
vcpkg install microsandbox
```

### Using Conan

```bash
conan install microsandbox/0.0.1
```

## Usage

```cpp
#include <iostream>
#include <microsandbox/microsandbox.hpp>

int main() {
    // Print a greeting
    std::string message = microsandbox::greet("World");
    std::cout << message << std::endl;
    return 0;
}
```

## License

[MIT](LICENSE)
