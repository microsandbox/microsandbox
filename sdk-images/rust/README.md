# Rust Docker Image

This directory contains a Dockerfile for setting up a Rust development environment.

## Features

- Rust 1.77.0
- Common development tools (rustfmt, clippy, rls, rust-analysis, rust-src)
- Cargo extensions (cargo-edit, cargo-watch, cargo-expand)
- Non-root user for better security

## Building the Image

To build the Docker image, run the following command from this directory:

```bash
docker build -t rust-dev .
```

## Running the Container

To start a container with the Rust environment, run:

```bash
docker run -it --rm -v $(pwd):/home/rust-user/work rust-dev bash
```

This will:

- Start a container with the Rust development image
- Mount your current directory to the work directory in the container
- Start an interactive bash session
- Remove the container when you exit the bash session

## Developing Rust Applications

Once inside the container, you can:

- Create new Rust projects with `cargo new my_project`
- Build projects with `cargo build`
- Run tests with `cargo test`
- Use development tools like `rustfmt` and `clippy`

## Customization

You can customize the Dockerfile to:

- Change the Rust version
- Add additional Cargo packages by modifying the `cargo install` command
- Add additional system dependencies by modifying the `apt-get install` command
- Change the user or security settings
