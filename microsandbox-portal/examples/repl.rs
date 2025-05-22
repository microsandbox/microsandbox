//! Example demonstrating the microsandbox-portal code execution in REPL environment.
//!
//! This example showcases the core functionality of the microsandbox-portal,
//! demonstrating code execution in REPL environments across multiple programming languages
//! in a sandboxed environment. It includes examples of:
//!
//! - Python code execution in REPL (when `python` feature is enabled)
//! - Node.js code execution in REPL (when `nodejs` feature is enabled)
//! - Stateful REPL sessions (maintaining state between executions)
//! - Error handling
//!
//! # Running the Example
//!
//! To run this example, use cargo with the desired language features enabled:
//!
//! ```bash
//! # Run with all languages enabled
//! cargo run --example repl --features "python nodejs"
//!
//! # Run with specific languages
//! cargo run --example repl --features "python"
//! cargo run --example repl --features "nodejs"
//! ```
//!
//! # Requirements
//!
//! Depending on which features you enable, you'll need:
//!
//! - Python: Python interpreter installed and available in PATH
//! - Node.js: Node.js installed and available in PATH
//!
//! # Example Output
//!
//! The example will output results from each language REPL execution, prefixed
//! with the output stream (Stdout/Stderr). For instance:
//!
//! ```text
//! ✅ Engines started successfully
//!
//! 🐍 Running Python example in REPL:
//! [Stdout] Factorial examples:
//! [Stdout] factorial(1) = 1
//! [Stdout] factorial(2) = 2
//! ...
//! ```
//!
//! # Note
//!
//! This example is designed to demonstrate basic usage of the microsandbox-portal.
//! In a real application, you might want to handle errors more gracefully and
//! implement more sophisticated code execution strategies in REPL environments.

use microsandbox_portal::portal::repl::start_engines;
#[cfg(any(feature = "python", feature = "nodejs"))]
use microsandbox_portal::portal::repl::Language;
use std::error::Error;

//--------------------------------------------------------------------------------------------------
// Functions: Main
//--------------------------------------------------------------------------------------------------

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Start the engines - this initializes all enabled engines
    let _engine_handle = start_engines().await?;
    println!("✅ Engines started successfully");

    // Example 1: Execute Python code in REPL
    #[cfg(feature = "python")]
    {
        println!("\n🐍 Running Python example in REPL:");
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

        let result = _engine_handle
            .eval(python_code, Language::Python, "123", Some(60))
            .await?;

        // Print the output
        for line in result {
            println!("[{:?}] {}", line.stream, line.text);
        }
    }

    // Example 2: Execute Node.js code in REPL
    #[cfg(feature = "nodejs")]
    {
        println!("\n🟨 Running Node.js example in REPL:");
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

        let result = _engine_handle
            .eval(javascript_code, Language::Node, "123", Some(60))
            .await?;

        // Print the output
        for line in result {
            println!("[{:?}] {}", line.stream, line.text);
        }
    }

    // Example 4: Stateful REPL session with Python
    #[cfg(feature = "python")]
    {
        println!("\n🔄 Python stateful REPL session example:");

        // First execution - define a variable
        let python_step1 = "x = 10";
        let result1 = _engine_handle
            .eval(python_step1, Language::Python, "123", None)
            .await?;
        for line in result1 {
            println!("[{:?}] {}", line.stream, line.text);
        }

        // Second execution - use the variable defined in the first step
        let python_step2 = "print(f'The value of x is {x}')";
        let result2 = _engine_handle
            .eval(python_step2, Language::Python, "123", None)
            .await?;
        for line in result2 {
            println!("[{:?}] {}", line.stream, line.text);
        }
    }

    // Example 5: Stateful REPL session with Node.js
    #[cfg(feature = "nodejs")]
    {
        println!("\n🔄 Node.js stateful REPL session example:");

        // First execution - define a variable
        let nodejs_step1 = "const greeting = 'Hello from JavaScript!';";
        let result1 = _engine_handle
            .eval(nodejs_step1, Language::Node, "123", None)
            .await?;
        for line in result1 {
            println!("[{:?}] {}", line.stream, line.text);
        }

        // Second execution - use the variable defined in the first step
        let nodejs_step2 = "console.log(greeting);";
        let result2 = _engine_handle
            .eval(nodejs_step2, Language::Node, "123", None)
            .await?;
        for line in result2 {
            println!("[{:?}] {}", line.stream, line.text);
        }
    }

    println!("\nExample completed successfully!");
    Ok(())
}
