# Rust SDK Image

This directory contains the Dockerfile for the Rust SDK image used with microsandbox.

## Features

- Rust 1.86.0 (Latest stable version)
- Common Rust development tools (rustfmt, clippy, rls, rust-analysis, rust-src)
- Cargo extensions (cargo-edit, cargo-watch, cargo-expand)
- microsandbox-portal service with Rust REPL support
- Non-root user for improved security

## Building the Image

To build the image, run the following command from the project root:

```bash
docker build -t msb-rust -f sdk-images/rust/Dockerfile .
```

The Dockerfile uses a multi-stage build that automatically compiles the portal binary with Rust features enabled, so no separate build step is required.

Alternatively, you can use the provided build script:

```bash
./scripts/build_sdk_images.sh -s rust
```

## Running the Container

To run the container with the portal service accessible on port 4444:

```bash
docker run -it -p 4444:4444 -e RUST_LOG=info --name msb-rust msb-rust
```

### Options

- `-p 4444:4444`: Maps container port 4444 to host port 4444
- `-e RUST_LOG=info`: Sets logging level for better debugging
- `--name msb-rust`: Names the container for easier reference

## Accessing the Container

To access a shell inside the running container:

```bash
docker exec -it msb-rust bash
```

## Stopping and Cleaning Up

```bash
# Stop the container
docker stop msb-rust

# Remove the container
docker rm msb-rust

# Remove the image (optional)
docker rmi msb-rust
```

## Customization

### Adding Additional Rust Packages

You can customize the Dockerfile to include additional Rust packages:

```dockerfile
# Add this to the Dockerfile
RUN cargo install \
    cargo-deny \
    cargo-audit \
    cargo-outdated
```

### Mounting Local Files

To access your local files inside the container:

```bash
docker run -it -p 4444:4444 -v $(pwd)/your_code:/home/rust-user/work --name msb-rust msb-rust
```

## Troubleshooting

If you encounter connection issues to the portal:

1. Check the logs: `docker logs msb-rust`
2. Verify the portal is running: `docker exec -it msb-rust ps aux | grep portal`
3. Ensure port 4444 is available on your host machine
