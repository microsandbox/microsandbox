from conan import ConanFile
from conan.tools.cmake import CMake, cmake_layout, CMakeToolchain

class MicrosandboxConan(ConanFile):
    name = "microsandbox"
    version = "0.1.0"
    license = "MIT"  # Update with your actual license
    author = "Steve Akinyemi"  # Update with your name
    url = "https://github.com/yourusername/monocore"  # Update with your actual repo URL
    description = "Microsandbox C++ library for improved security through sandboxing"
    topics = ("sandbox", "security", "isolation")

    # Binary configuration
    settings = "os", "compiler", "build_type", "arch"
    options = {"shared": [True, False], "fPIC": [True, False]}
    default_options = {"shared": False, "fPIC": True}

    exports_sources = "CMakeLists.txt", "src/*", "include/*", "test/*", "cmake/*"

    def config_options(self):
        if self.settings.os == "Windows":
            del self.options.fPIC

    def layout(self):
        cmake_layout(self)

    def generate(self):
        tc = CMakeToolchain(self)
        tc.generate()

    def build(self):
        cmake = CMake(self)
        cmake.configure()
        cmake.build()

    def package(self):
        cmake = CMake(self)
        cmake.install()

    def package_info(self):
        self.cpp_info.libs = ["microsandbox"]
        self.cpp_info.includedirs = ["include"]
