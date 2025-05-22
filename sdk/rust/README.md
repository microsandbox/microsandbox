# Microsandbox Rust SDK

A Rust SDK for microsandbox - secure self-hosted sandboxes for executing untrusted code. This SDK allows you to create isolated environments for running code with controlled access to system resources.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
microsandbox = "0.1.0"
tokio = { version = "1", features = ["full"] }
```

## Usage

### Simple Python Sandbox Example

```rust
use microsandbox::{SandboxOptions, PythonSandbox};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a Python sandbox
    let mut sb = PythonSandbox::create(SandboxOptions::builder().name("test").build()).await?;

    // Run Python code
    let exec = sb.run(r#"name = "Python""#).await?;
    let exec = sb.run(r#"print(f"Hello {name}!")"#).await?;

    // Get the output
    println!("{}", exec.output().await?); // prints Hello Python!

    // Stop the sandbox
    sb.stop().await?;

    Ok(())
}
```

### Using Environment Variables

The SDK will automatically read environment variables:

- `MSB_API_KEY` - Your API key for authentication
- `MSB_SERVER_URL` - The URL of the microsandbox server (defaults to http://127.0.0.1:5555)

You can also set these values programmatically:

```rust
let sb = PythonSandbox::create(
    SandboxOptions::builder()
        .name("test")
        .api_key("msb_your_api_key")
        .server_url("http://your-server-url:5555")
        .build()
).await?;
```

## Features

- **Python Sandbox** - Run Python code in a secure sandbox
- **Shell Commands** - Execute shell commands in the sandbox
- **Fully Async** - Built with modern async/await support

## Examples

Check out the [examples directory](./examples) for sample scripts that demonstrate how to:

- Run Python code in a sandbox
- Execute shell commands
- Monitor resource usage
- Configure sandbox options

## License

[Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0)
