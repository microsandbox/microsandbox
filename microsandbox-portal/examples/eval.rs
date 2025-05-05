//! Example demonstrating the microsandbox-portal code evaluation system.
//!
//! This example showcases the core functionality of the microsandbox-portal,
//! demonstrating code evaluation across multiple programming languages in a sandboxed
//! environment. It includes examples of:
//!
//! - Rust code evaluation (when `rust` feature is enabled)
//! - Python code evaluation (when `python` feature is enabled)
//! - Node.js code evaluation (when `nodejs` feature is enabled)
//! - Stateful evaluation (maintaining state between evaluations)
//! - Error handling
//!
//! # Running the Example
//!
//! To run this example, use cargo with the desired language features enabled:
//!
//! ```bash
//! # Run with all languages enabled
//! cargo run --example eval --features "python nodejs rust"
//!
//! # Run with specific languages
//! cargo run --example eval --features "python rust"
//! cargo run --example eval --features "nodejs"
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
//! The example will output results from each language evaluation, prefixed
//! with the output stream (Stdout/Stderr). For instance:
//!
//! ```text
//! ✅ Engines started successfully
//!
//! 🦀 Running Rust example:
//! [Stdout] Fibonacci sequence:
//! [Stdout] fib(0) = 0
//! [Stdout] fib(1) = 1
//! ...
//! ```
//!
//! # Note
//!
//! This example is designed to demonstrate basic usage of the microsandbox-portal.
//! In a real application, you might want to handle errors more gracefully and
//! implement more sophisticated code evaluation strategies.

use microsandbox_portal::code::{start_engines, Language};
use std::error::Error;

//--------------------------------------------------------------------------------------------------
// Functions: Main
//--------------------------------------------------------------------------------------------------

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Start the engines - this initializes all enabled engines
    let engine_handle = start_engines().await?;
    println!("✅ Engines started successfully");

        // Example 1: Evaluate Rust code
        #[cfg(feature = "rust")]
        {
            println!("\n🦀 Running Rust example:");
            let rust_code = r#"
    // Define a function
    fn fibonacci(n: u32) -> u32 {
        match n {
            0 => 0,
            1 => 1,
            _ => fibonacci(n-1) + fibonacci(n-2),
        }
    }

    // Use the function
    println!("Fibonacci sequence:");
    for i in 0..10 {
        println!("fib({}) = {}", i, fibonacci(i));
    }
            "#;

            let result = engine_handle.eval(rust_code, Language::Rust).await?;

            // Print the output
            for line in result {
                println!("[{:?}] {}", line.stream, line.text);
            }
        }

    // Example 2: Evaluate Python code
    #[cfg(feature = "python")]
    {
        println!("\n🐍 Running Python example:");
        let python_code = r#"
# Define a function
def factorial(n):
    if n == 0 or n == 1:
        return 1
    else:
        return n * factorial(n-1)

# Use the function
print("Factorial examples:")
for i in range(1, 6):
    print(f"factorial({i}) = {factorial(i)}")

# Create a simple data structure
fruits = ["apple", "banana", "cherry"]
print("\nFruit list:")
for i, fruit in enumerate(fruits):
    print(f"{i+1}. {fruit}")
        "#;

        let result = engine_handle.eval(python_code, Language::Python).await?;

        // Print the output
        for line in result {
            println!("[{:?}] {}", line.stream, line.text);
        }
    }

    // Example 3: Evaluate Node.js code
    #[cfg(feature = "nodejs")]
    {
        println!("\n🟨 Running Node.js example:");
        let javascript_code = r#"
// Define a class
class Person {
    constructor(name, age) {
        this.name = name;
        this.age = age;
    }

    greet() {
        return `Hello, my name is ${this.name} and I am ${this.age} years old.`;
    }
}

// Use the class
const people = [
    new Person("Alice", 28),
    new Person("Bob", 32),
    new Person("Charlie", 22)
];

console.log("People greetings:");
people.forEach(person => {
    console.log(person.greet());
});

// Demonstrate async functionality
console.log("\nAsync example:");
async function fetchData() {
    // Simulate fetching data
    return new Promise(resolve => {
        setTimeout(() => {
            resolve({ success: true, data: [1, 2, 3, 4, 5] });
        }, 500);
    });
}

// We can't actually wait for this in a REPL, but we can start it
fetchData().then(result => {
    console.log("Data fetched:", result);
});

console.log("Waiting for data...");
        "#;

        let result = engine_handle.eval(javascript_code, Language::Node).await?;

        // Print the output
        for line in result {
            println!("[{:?}] {}", line.stream, line.text);
        }
    }

    // Example 4: Stateful evaluation with Python
    #[cfg(feature = "python")]
    {
        println!("\n🔄 Python stateful evaluation example:");

        // First evaluation - define a variable
        let python_step1 = "x = 10";
        let result1 = engine_handle.eval(python_step1, Language::Python).await?;
        for line in result1 {
            println!("[{:?}] {}", line.stream, line.text);
        }

        // Second evaluation - use the variable defined in the first step
        let python_step2 = "print(f'x = {x}')\nx += 5\nprint(f'x + 5 = {x}')";
        let result2 = engine_handle.eval(python_step2, Language::Python).await?;
        for line in result2 {
            println!("[{:?}] {}", line.stream, line.text);
        }
    }

    // Example 5: Stateful evaluation with Node.js
    #[cfg(feature = "nodejs")]
    {
        println!("\n🔄 Node.js stateful evaluation example:");

        // First evaluation - define a variable
        let nodejs_step1 = "let counter = 10;";
        let result1 = engine_handle.eval(nodejs_step1, Language::Node).await?;
        for line in result1 {
            println!("[{:?}] {}", line.stream, line.text);
        }

        // Second evaluation - use the variable defined in the first step
        let nodejs_step2 = "console.log(`counter = ${counter}`); counter += 5; console.log(`counter + 5 = ${counter}`);";
        let result2 = engine_handle.eval(nodejs_step2, Language::Node).await?;
        for line in result2 {
            println!("[{:?}] {}", line.stream, line.text);
        }
    }

    // Example 6: Stateful evaluation with Rust
    #[cfg(feature = "rust")]
    {
        println!("\n🔄 Rust stateful evaluation example:");

        // First evaluation - define a variable
        let rust_step1 = "let mut counter = 10;";
        let result1 = engine_handle.eval(rust_step1, Language::Rust).await?;
        for line in result1 {
            println!("[{:?}] {}", line.stream, line.text);
        }

        // Second evaluation - use the variable defined in the first step
        let rust_step2 = "println!(\"counter = {}\", counter);\ncounter += 5;\nprintln!(\"counter + 5 = {}\", counter);";
        let result2 = engine_handle.eval(rust_step2, Language::Rust).await?;
        for line in result2 {
            println!("[{:?}] {}", line.stream, line.text);
        }
    }

    // Shutdown the engines
    engine_handle.shutdown().await?;
    println!("\n✅ Engines shut down successfully");

    Ok(())
}
