/// Simple example showing how to use the microsandbox Rust crate.
///
/// Before running this example:
///   1. Install the crate: cargo add microsandbox
///   2. Run this example: cargo run --example hello_example
// Import the microsandbox crate
use microsandbox;

fn main() {
    // Call the greet function
    microsandbox::greet("World");
}
