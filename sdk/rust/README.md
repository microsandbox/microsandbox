# Microsandbox Rust SDK

A minimal Rust SDK for the Microsandbox project.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
microsandbox = "0.0.1"
```

## Usage

```rust
use microsandbox::greet;

fn main() {
    // Print a greeting
    greet("World");
}
```

## Examples

Check out the [examples directory](./examples) for sample scripts that demonstrate how to:

- Use the basic features of the SDK
- Run simple applications using the crate

## License

[Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0)
