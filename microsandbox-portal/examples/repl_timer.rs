//! Example demonstrating REPL execution with timeouts.
//!
//! This example showcases how to use the microsandbox-portal to execute code in REPL
//! environments with a fixed timeout of 10 seconds. It demonstrates:
//!
//! - Executing code in different language REPLs (Python, Node.js, Rust)
//! - Setting a fixed timeout for REPL execution
//! - Handling timeout scenarios with infinite loops or long computations
//! - Processing execution output and checking for timeout errors
//!
//! # Running the Example
//!
//! To run this example, use cargo with the desired language features enabled:
//!
//! ```bash
//! # Run with all languages enabled
//! cargo run --example repl_timer --features "python nodejs rust"
//!
//! # Run with specific languages
//! cargo run --example repl_timer --features "python rust"
//! cargo run --example repl_timer --features "nodejs"
//! ```
//!
//! # Requirements
//!
//! Depending on which features you enable, you'll need:
//!
//! - Python: Python interpreter installed and available in PATH
//! - Node.js: Node.js installed and available in PATH
//! - Rust: No additional requirements (uses evcxr)
//!
//! # Example Output
//!
//! The example will execute code in each enabled language REPL with a 10-second timeout:
//!
//! ```text
//! 🕒 Running examples with 10-second timeout
//!
//! 🐍 Python normal execution:
//! [Stdout] Counting from 1 to 5 with delays:
//! [Stdout] Count: 1
//! [Stdout] Count: 2
//! [Stdout] Count: 3
//! [Stdout] Count: 4
//! [Stdout] Count: 5
//! [Stdout] Python counting complete!
//!
//! 🐍 Python infinite loop (should timeout):
//! [Stderr] Execution timed out after 10 seconds
//! ```

use microsandbox_portal::portal::repl::{start_engines, Language};
use std::error::Error;

/// Fixed timeout duration for all executions in this example
const TIMEOUT_SECONDS: u64 = 10;

/// Print output lines from a REPL execution with a prefix
fn print_output(prefix: &str, lines: &[microsandbox_portal::portal::repl::Line]) {
    println!("{} execution result:", prefix);
    if lines.is_empty() {
        println!("No output produced");
    } else {
        for line in lines {
            println!("[{:?}] {}", line.stream, line.text);
        }
    }
    println!();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Start the engines - this initializes all enabled engines
    let engine_handle = start_engines().await?;
    println!("✅ Engines started successfully\n");
    println!(
        "🕒 Running examples with {}-second timeout\n",
        TIMEOUT_SECONDS
    );

    // Example 1: Python code that completes normally
    #[cfg(feature = "python")]
    {
        println!("🐍 Python normal execution:");
        let python_normal = r#"
import time

print("Counting from 1 to 5 with delays:")
for i in range(1, 6):
    print(f"Count: {i}")
    time.sleep(1)  # Sleep for 1 second

print("Python counting complete!")
        "#;

        let result = engine_handle
            .eval(
                python_normal,
                Language::Python,
                "python_normal",
                Some(TIMEOUT_SECONDS),
            )
            .await?;

        print_output("Python normal", &result);
    }

    // Example 2: Python code that runs in an infinite loop (should timeout)
    #[cfg(feature = "python")]
    {
        println!("🐍 Python infinite loop (should timeout):");
        let python_infinite = r#"
import time

print("Starting infinite loop in Python...")
counter = 0
while True:
    counter += 1
    print(f"Iteration {counter}...")
    time.sleep(0.5)  # Sleep for half a second
        "#;

        let result = engine_handle
            .eval(
                python_infinite,
                Language::Python,
                "python_infinite",
                Some(TIMEOUT_SECONDS),
            )
            .await?;

        print_output("Python infinite loop", &result);
    }

    // Example 3: Node.js code that completes normally
    #[cfg(feature = "nodejs")]
    {
        println!("🟨 Node.js normal execution:");
        let js_normal = r#"
console.log("Counting from 1 to 5 with delays:");

// Use a simple for loop with a delay function for consistency with Python example
function sleep(ms) {
  const start = new Date().getTime();
  while (new Date().getTime() < start + ms);
}

for (let i = 1; i <= 5; i++) {
  console.log(`Count: ${i}`);
  sleep(1000); // Sleep for 1 second
}

console.log("Node.js counting complete!");
        "#;

        let result = engine_handle
            .eval(
                js_normal,
                Language::Node,
                "js_normal",
                Some(TIMEOUT_SECONDS),
            )
            .await?;

        print_output("Node.js normal", &result);
    }

    // Example 4: Node.js code that runs in an infinite loop (should timeout)
    #[cfg(feature = "nodejs")]
    {
        println!("🟨 Node.js infinite loop (should timeout):");
        let js_infinite = r#"
console.log("Starting infinite loop in Node.js...");

// Use a while loop for consistency with Python example
function sleep(ms) {
  const start = new Date().getTime();
  while (new Date().getTime() < start + ms);
}

let counter = 0;
while (true) {
  counter++;
  console.log(`Iteration ${counter}...`);
  sleep(500); // Sleep for half a second
}
        "#;

        let result = engine_handle
            .eval(
                js_infinite,
                Language::Node,
                "js_infinite",
                Some(TIMEOUT_SECONDS),
            )
            .await?;

        print_output("Node.js infinite loop", &result);
    }

    //     // Example 5: Rust code that completes normally
    //     #[cfg(feature = "rust")]
    //     {
    //         println!("🦀 Rust normal execution:");
    //         let rust_normal = r#"
    // use std::thread::sleep;
    // use std::time::Duration;

    // println!("Rust timer example:");
    // for i in 1..=5 {
    //     println!("Second {}", i);
    //     sleep(Duration::from_secs(1));
    // }
    // println!("Rust timer complete!");
    //         "#;

    //         let result = engine_handle
    //             .eval(
    //                 rust_normal,
    //                 Language::Rust,
    //                 "rust_normal",
    //                 Some(TIMEOUT_SECONDS),
    //             )
    //             .await?;

    //         print_output("Rust normal", &result);
    //     }

    //     // Example 6: Rust code with a long computation (should timeout)
    //     #[cfg(feature = "rust")]
    //     {
    //         println!("🦀 Rust long computation (should timeout):");
    //         let rust_long = r#"
    // println!("Computing Fibonacci numbers recursively (inefficient algorithm):");

    // // Intentionally inefficient recursive Fibonacci that will time out
    // fn fibonacci(n: u64) -> u64 {
    //     if n <= 1 {
    //         return n;
    //     }
    //     fibonacci(n - 1) + fibonacci(n - 2)
    // }

    // // This will take much longer than our timeout
    // println!("Starting computation of fibonacci(50)...");
    // let result = fibonacci(50);
    // println!("Result: {}", result);  // This line should never be reached
    //         "#;

    //         let result = engine_handle
    //             .eval(
    //                 rust_long,
    //                 Language::Rust,
    //                 "rust_long",
    //                 Some(TIMEOUT_SECONDS),
    //             )
    //             .await?;

    //         print_output("Rust long computation", &result);
    //     }

    println!("✅ All examples completed!");
    Ok(())
}
