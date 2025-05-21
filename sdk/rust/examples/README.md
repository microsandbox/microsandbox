# Microsandbox Rust SDK Examples

This directory contains example scripts to demonstrate how to use the Microsandbox Rust SDK.

## Available Examples

### 1. Simple Python Sandbox Example (`python_simple.rs`)

A basic example showing how to create a Python sandbox, run code, and handle the results.

```bash
# Run the example
cargo run --example python_simple
```

### 2. REPL Examples (`repl.rs`)

Advanced examples demonstrating different ways to create, manage, and use Python sandboxes.

```bash
# Run the example
cargo run --example repl
```

### 3. Command Execution Examples (`command.rs`)

Examples showing how to execute shell commands in a sandbox environment.

```bash
# Run the example
cargo run --example command
```

### 4. Metrics Monitoring Examples (`metrics.rs`)

Examples demonstrating how to retrieve and monitor resource usage metrics from a sandbox.

```bash
# Run the example
cargo run --example metrics
```

### 5. Node.js Sandbox Examples (`node.rs`)

Examples showing how to use NodeSandbox to execute JavaScript code.

```bash
# Run the example
cargo run --example node
```

## Contents

The examples demonstrate:

- Creating Python and Node.js sandboxes
- Running code and handling outputs
- Executing shell commands
- Monitoring resource usage
- Error handling
- State persistence between executions
- Working with timeouts and resource limits

## Environment Setup

Before running the examples, make sure you have:

1. Set up your API key as an environment variable (or in an `.env` file):

   ```
   MSB_API_KEY=msb_your_api_key
   ```

2. Installed the required dependencies:

   ```toml
   [dependencies]
   microsandbox = "0.1.0"
   tokio = { version = "1", features = ["full"] }
   futures = "0.3"
   ```

3. Started the Microsandbox server (local or remote)

## Running Examples

```bash
# Run from the SDK directory
cargo run --example <example_name>

# For example, to run the Node.js example:
cargo run --example node
```

## Notes

- The examples assume you have Rust and Cargo installed
- Examples require a running Microsandbox server (local or remote)
- For more information on Cargo, see the [Cargo Book](https://doc.rust-lang.org/cargo/)
