---
icon: download
title: Installation
description: Install the Microsandbox CLI and set up your environment
order: 101
---

# Installation

This guide covers installing the Microsandbox CLI (`msb`) and setting up your development environment.

## System Requirements

### Platform Support

| Platform    | Status         | Requirements                                                             |
| ----------- | -------------- | ------------------------------------------------------------------------ |
| **macOS**   | ✅ Supported   | Apple Silicon (M1/M2/M3/M4)                                              |
| **Linux**   | ✅ Supported   | KVM virtualization enabled                                               |
| **Windows** | 🚧 Coming Soon | [Track progress](https://github.com/microsandbox/microsandbox/issues/47) |

### Hardware Requirements

- **Memory**: Minimum 4GB RAM (8GB+ recommended)
- **Storage**: At least 2GB free space for images
- **CPU**: Modern multi-core processor with virtualization support

<div align='center'>• • •</div>

## Install the CLI

### Quick Install (Recommended)

The fastest way to install the Microsandbox CLI:

```bash
curl -sSL https://get.microsandbox.dev | sh
```

This script will:

- Download the latest version for your platform
- Install the `msb` binary to your PATH
- Set up shell completions (optional)

### Manual Installation

#### Download from GitHub Releases

1. Visit the [releases page](https://github.com/microsandbox/microsandbox/releases)
2. Download the appropriate binary for your platform:
   - `microsandbox-macos-aarch64.tar.gz` (Apple Silicon Mac)
   - `microsandbox-linux-x86_64.tar.gz` (Linux x64)
3. Extract and move to your PATH:

```bash
tar -xzf microsandbox-*.tar.gz
sudo mv msb /usr/local/bin/
```

#### Build from Source

If you have Rust installed:

```bash
git clone https://github.com/microsandbox/microsandbox.git
cd microsandbox
cargo build --release
sudo cp target/release/msb /usr/local/bin/
```

<div align='center'>• • •</div>

## Verify Installation

Check that the CLI is installed correctly:

```bash
msb --version
```

You should see output similar to:

```
msb (microsandbox) 0.1.0
```

<div align='center'>• • •</div>

## Platform-Specific Setup

### macOS Setup

1. **Enable Virtualization**: Modern Macs have this enabled by default
2. **Install Xcode Command Line Tools** (if not already installed):
   ```bash
   xcode-select --install
   ```

### Linux Setup

1. **Check KVM Support**:

   ```bash
   # Check if KVM is available
   lsmod | grep kvm

   # Check if your CPU supports virtualization
   egrep -c '(vmx|svm)' /proc/cpuinfo
   ```

2. **Install KVM** (Ubuntu/Debian):

   ```bash
   sudo apt update
   sudo apt install qemu-kvm libvirt-daemon-system libvirt-clients bridge-utils
   ```

3. **Add User to KVM Group**:

   ```bash
   sudo usermod -aG kvm $USER
   sudo usermod -aG libvirt $USER
   # Log out and back in for changes to take effect
   ```

4. **Verify KVM Access**:
   ```bash
   # Should show /dev/kvm with proper permissions
   ls -la /dev/kvm
   ```

<div align='center'>• • •</div>

## Shell Completions

Enable tab completion for the `msb` command:

+++ Bash

```bash
echo 'eval "$(msb completion bash)"' >> ~/.bashrc
source ~/.bashrc
```

+++ Zsh

```bash
echo 'eval "$(msb completion zsh)"' >> ~/.zshrc
source ~/.zshrc
```

+++ Fish

```bash
msb completion fish > ~/.config/fish/completions/msb.fish
```

+++

<div align='center'>• • •</div>

## Environment Variables

Set up optional environment variables:

```bash
# Add to your shell profile (~/.bashrc, ~/.zshrc, etc.)

# Default server URL (optional)
export MSB_SERVER_URL="http://127.0.0.1:5555"

# API key for authentication (set after generating)
export MSB_API_KEY="msb_your_api_key_here"

# Default log level
export MSB_LOG_LEVEL="info"
```

<div align='center'>• • •</div>

## Troubleshooting

### Common Issues

#### "Command not found: msb"

- Ensure `/usr/local/bin` is in your PATH
- Try restarting your terminal
- Check installation location: `which msb`

#### "Permission denied" on Linux

- Ensure your user is in the `kvm` and `libvirt` groups
- Log out and back in after adding to groups
- Check `/dev/kvm` permissions

#### "Virtualization not supported"

- Verify your CPU supports virtualization
- Enable virtualization in BIOS/UEFI settings
- On macOS, ensure you're using Apple Silicon

### Getting Help

If you encounter issues:

1. Search [GitHub issues](https://github.com/microsandbox/microsandbox/issues)
2. Ask in our [Discord community](https://discord.gg/T95Y3XnEAK)

<div align='center'>• • •</div>

## What's Next?

Now that you have the CLI installed:

1. [**Set up self-hosting**](self-hosting.md) - Start your own server
2. [**Install an SDK**](../sdks/index.md) - Start building with sandboxes
3. [**See examples**](../examples/index.md) - Learn from practical examples

---

!!!tip
Run `msb --help` to see all available commands and options.
!!!
