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
# Installation Paths
# -----------------------------------------------------------------------------
HOME_LIB := $(HOME)/.local/lib
HOME_BIN := $(HOME)/.local/bin

# -----------------------------------------------------------------------------
# Build Paths and Directories
# -----------------------------------------------------------------------------
MSB_RELEASE_BIN := target/release/msb
MSBRUN_RELEASE_BIN := target/release/msbrun
MSBSERVER_RELEASE_BIN := target/release/msbserver
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

_build_msb: $(MSB_RELEASE_BIN) $(MSBRUN_RELEASE_BIN) $(MSBSERVER_RELEASE_BIN)
	@cp $(MSB_RELEASE_BIN) $(BUILD_DIR)/
	@cp $(MSBRUN_RELEASE_BIN) $(BUILD_DIR)/
	@cp $(MSBSERVER_RELEASE_BIN) $(BUILD_DIR)/
	@echo "Msb build artifacts copied to $(BUILD_DIR)/"

_build_aliases:
	@mkdir -p $(BUILD_DIR)
	@cp $(ALIASES_DIR)/msr $(BUILD_DIR)/
	@cp $(ALIASES_DIR)/msx $(BUILD_DIR)/
	@cp $(ALIASES_DIR)/msi $(BUILD_DIR)/
	@echo "Alias scripts copied to $(BUILD_DIR)/"

# -----------------------------------------------------------------------------
# Binary Building
# -----------------------------------------------------------------------------
$(MSB_RELEASE_BIN): build_libkrun
	cd microsandbox-core
ifeq ($(OS),Darwin)
	RUSTFLAGS="-C link-args=-Wl,-rpath,@executable_path/../lib,-rpath,@executable_path" cargo build --release --bin msb --features cli $(FEATURES)
else
	RUSTFLAGS="-C link-args=-Wl,-rpath,\$$ORIGIN/../lib,-rpath,\$$ORIGIN" cargo build --release --bin msb --features cli $(FEATURES)
endif

$(MSBRUN_RELEASE_BIN): build_libkrun
	cd microsandbox-core
ifeq ($(OS),Darwin)
	RUSTFLAGS="-C link-args=-Wl,-rpath,@executable_path/../lib,-rpath,@executable_path" cargo build --release --bin msbrun --features cli $(FEATURES)
	codesign --entitlements microsandbox.entitlements --force -s - $@
else
	RUSTFLAGS="-C link-args=-Wl,-rpath,\$$ORIGIN/../lib,-rpath,\$$ORIGIN" cargo build --release --bin msbrun --features cli $(FEATURES)
endif

$(MSBSERVER_RELEASE_BIN): build_libkrun
	cd microsandbox-core
ifeq ($(OS),Darwin)
	RUSTFLAGS="-C link-args=-Wl,-rpath,@executable_path/../lib,-rpath,@executable_path" cargo build --release --bin msbserver --features cli $(FEATURES)
else
	RUSTFLAGS="-C link-args=-Wl,-rpath,\$$ORIGIN/../lib,-rpath,\$$ORIGIN" cargo build --release --bin msbserver --features cli $(FEATURES)
endif

# -----------------------------------------------------------------------------
# Installation
# -----------------------------------------------------------------------------
install: build
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
