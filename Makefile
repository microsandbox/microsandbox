# =============================================================================
# Microsandbox Makefile - Build, install, and run microsandbox components
# =============================================================================

# -----------------------------------------------------------------------------
# System Detection and Architecture
# -----------------------------------------------------------------------------
OS := $(shell uname -s)
ARCH := $(shell uname -m)
ifeq ($(ARCH),aarch64)
	ARCH := arm64
endif
ifeq ($(ARCH),x86_64)
	ARCH := x86_64
endif

# -----------------------------------------------------------------------------
# Build Configuration
# -----------------------------------------------------------------------------
BUILD_TYPE ?= release
CARGO_BUILD_MODE := $(if $(filter debug,$(BUILD_TYPE)),,--release)
CARGO_TARGET_DIR := target/$(if $(filter debug,$(BUILD_TYPE)),debug,release)

# -----------------------------------------------------------------------------
# Installation Paths
# -----------------------------------------------------------------------------
HOME_LIB := $(HOME)/.local/lib
HOME_BIN := $(HOME)/.local/bin

# -----------------------------------------------------------------------------
# Build Paths and Directories
# -----------------------------------------------------------------------------
MSB_BIN := $(CARGO_TARGET_DIR)/msb
MSBRUN_BIN := $(CARGO_TARGET_DIR)/msbrun
MSBSERVER_BIN := $(CARGO_TARGET_DIR)/msbserver
EXAMPLES_DIR := target/release/examples
BENCHES_DIR := target/release
BUILD_DIR := build
SCRIPT_DIR := scripts
ALIASES_DIR := $(SCRIPT_DIR)/aliases

# -----------------------------------------------------------------------------
# Library Detection
# -----------------------------------------------------------------------------
ifeq ($(OS),Darwin)
	LIBKRUNFW_FILE := $(shell ls $(BUILD_DIR)/libkrunfw.*.dylib 2>/dev/null | head -n1)
	LIBKRUN_FILE := $(shell ls $(BUILD_DIR)/libkrun.*.dylib 2>/dev/null | head -n1)
else
	LIBKRUNFW_FILE := $(shell ls $(BUILD_DIR)/libkrunfw.so.* 2>/dev/null | head -n1)
	LIBKRUN_FILE := $(shell ls $(BUILD_DIR)/libkrun.so.* 2>/dev/null | head -n1)
endif

# -----------------------------------------------------------------------------
# Phony Targets Declaration
# -----------------------------------------------------------------------------
.PHONY: all build install clean build_libkrun example bench bin _run_example _run_bench _run_bin help uninstall microsandbox _build_aliases

# -----------------------------------------------------------------------------
# Main Targets
# -----------------------------------------------------------------------------
all: build

build: build_libkrun
	@$(MAKE) _build_msb
	@$(MAKE) _build_aliases

_build_msb: $(MSB_BIN) $(MSBRUN_BIN) $(MSBSERVER_BIN)
	@mkdir -p $(BUILD_DIR)
	@cp $(MSB_BIN) $(BUILD_DIR)/
	@cp $(MSBRUN_BIN) $(BUILD_DIR)/
	@cp $(MSBSERVER_BIN) $(BUILD_DIR)/
	@echo "Msb build artifacts ($(BUILD_TYPE) mode) copied to $(BUILD_DIR)/"

_build_aliases:
	@mkdir -p $(BUILD_DIR)
	@cp $(ALIASES_DIR)/msr $(BUILD_DIR)/
	@cp $(ALIASES_DIR)/msx $(BUILD_DIR)/
	@cp $(ALIASES_DIR)/msi $(BUILD_DIR)/
	@echo "Alias scripts copied to $(BUILD_DIR)/"

# -----------------------------------------------------------------------------
# Binary Building
# -----------------------------------------------------------------------------
$(MSB_BIN): build_libkrun
	cd microsandbox-core
ifeq ($(OS),Darwin)
	RUSTFLAGS="-C link-args=-Wl,-rpath,@executable_path/../lib,-rpath,@executable_path" cargo build $(CARGO_BUILD_MODE) --bin msb --features cli $(FEATURES)
else
	RUSTFLAGS="-C link-args=-Wl,-rpath,\$$ORIGIN/../lib,-rpath,\$$ORIGIN" cargo build $(CARGO_BUILD_MODE) --bin msb --features cli $(FEATURES)
endif

$(MSBRUN_BIN): build_libkrun
	cd microsandbox-core
ifeq ($(OS),Darwin)
	RUSTFLAGS="-C link-args=-Wl,-rpath,@executable_path/../lib,-rpath,@executable_path" cargo build $(CARGO_BUILD_MODE) --bin msbrun --features cli $(FEATURES)
	codesign --entitlements microsandbox.entitlements --force -s - $@
else
	RUSTFLAGS="-C link-args=-Wl,-rpath,\$$ORIGIN/../lib,-rpath,\$$ORIGIN" cargo build $(CARGO_BUILD_MODE) --bin msbrun --features cli $(FEATURES)
endif

$(MSBSERVER_BIN): build_libkrun
	cd microsandbox-core
ifeq ($(OS),Darwin)
	RUSTFLAGS="-C link-args=-Wl,-rpath,@executable_path/../lib,-rpath,@executable_path" cargo build $(CARGO_BUILD_MODE) --bin msbserver --features cli $(FEATURES)
else
	RUSTFLAGS="-C link-args=-Wl,-rpath,\$$ORIGIN/../lib,-rpath,\$$ORIGIN" cargo build $(CARGO_BUILD_MODE) --bin msbserver --features cli $(FEATURES)
endif

# -----------------------------------------------------------------------------
# Installation
# -----------------------------------------------------------------------------
install: build
	@echo "Installing $(BUILD_TYPE) build..."
	install -d $(HOME_BIN)
	install -d $(HOME_LIB)
	install -m 755 $(BUILD_DIR)/msb $(HOME_BIN)/msb
	install -m 755 $(BUILD_DIR)/msbrun $(HOME_BIN)/msbrun
	install -m 755 $(BUILD_DIR)/msbserver $(HOME_BIN)/msbserver
	install -m 755 $(BUILD_DIR)/msr $(HOME_BIN)/msr
	install -m 755 $(BUILD_DIR)/msx $(HOME_BIN)/msx
	install -m 755 $(BUILD_DIR)/msi $(HOME_BIN)/msi
	@if [ -n "$(LIBKRUNFW_FILE)" ]; then \
		install -m 755 $(LIBKRUNFW_FILE) $(HOME_LIB)/; \
		cd $(HOME_LIB) && ln -sf $(notdir $(LIBKRUNFW_FILE)) libkrunfw.dylib; \
	else \
		echo "Warning: libkrunfw library not found in build directory"; \
	fi
	@if [ -n "$(LIBKRUN_FILE)" ]; then \
		install -m 755 $(LIBKRUN_FILE) $(HOME_LIB)/; \
		cd $(HOME_LIB) && ln -sf $(notdir $(LIBKRUN_FILE)) libkrun.dylib; \
	else \
		echo "Warning: libkrun library not found in build directory"; \
	fi
	@echo "Installation of $(BUILD_TYPE) build complete."

# -----------------------------------------------------------------------------
# Maintenance
# -----------------------------------------------------------------------------
clean:
	rm -rf $(BUILD_DIR)
	cd microsandbox-core && cargo clean && rm -rf build

uninstall:
	rm -f $(HOME_BIN)/msb
	rm -f $(HOME_BIN)/msbrun
	rm -f $(HOME_BIN)/msbserver
	rm -f $(HOME_BIN)/msr
	rm -f $(HOME_BIN)/msx
	rm -f $(HOME_BIN)/msi
	rm -f $(HOME_LIB)/libkrunfw.dylib
	rm -f $(HOME_LIB)/libkrun.dylib
	@if [ -n "$(LIBKRUNFW_FILE)" ]; then \
		rm -f $(HOME_LIB)/$(notdir $(LIBKRUNFW_FILE)); \
	fi
	@if [ -n "$(LIBKRUN_FILE)" ]; then \
		rm -f $(HOME_LIB)/$(notdir $(LIBKRUN_FILE)); \
	fi

build_libkrun:
	./scripts/build_libkrun.sh --no-clean

# Catch-all target to allow example names and arguments
%:
	@:

# -----------------------------------------------------------------------------
# Help Documentation
# -----------------------------------------------------------------------------
help:
	@echo "Microsandbox Makefile Help"
	@echo "======================"
	@echo
	@echo "Main Targets:"
	@echo "  make build                  - Build microsandbox components"
	@echo "  make install                - Install binaries and libraries to ~/.local/{bin,lib}"
	@echo "  make uninstall              - Remove all installed components"
	@echo "  make clean                  - Remove build artifacts"
	@echo "  make build_libkrun          - Build libkrun dependency"
	@echo
	@echo "Note: For commands that accept arguments, use -- to separate them"
	@echo "      from the make target name."
