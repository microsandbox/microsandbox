package main

import (
	"fmt"
	"log"

	msb "github.com/keithang/microsandbox/sdk/go"
)

// contextManagerEquivalentExample demonstrates the Go equivalent of Python's context manager pattern.
func contextManagerEquivalentExample() {
	fmt.Println("\n=== Context Manager Equivalent Example ===")

	// Create a sandbox (equivalent to async with PythonSandbox.create())
	sandbox := msb.NewPythonSandbox(
		msb.WithName("sandbox-cm"),
	)

	// Start the sandbox
	if err := sandbox.Start("", 512, 1); err != nil {
		log.Fatalf("Failed to start sandbox: %v", err)
	}
	// Use defer for automatic cleanup (Go's equivalent of context manager)
	defer func() {
		if err := sandbox.Stop(); err != nil {
			log.Printf("Failed to stop sandbox: %v", err)
		}
	}()

	// Run some computation
	code := `print("Hello, world!")`
	execution, err := sandbox.Code().Run(code)
	if err != nil {
		log.Fatalf("Failed to run code: %v", err)
	}

	if output, err := execution.GetOutput(); err != nil {
		log.Printf("Failed to get output: %v", err)
	} else {
		fmt.Printf("Output: %s\n", output)
	}
}

// explicitLifecycleExample demonstrates explicit lifecycle management.
func explicitLifecycleExample() {
	fmt.Println("\n=== Explicit Lifecycle Example ===")

	// Create sandbox with custom configuration
	sandbox := msb.NewPythonSandbox(
		msb.WithServerUrl("http://127.0.0.1:5555"),
		msb.WithName("sandbox-explicit"),
	)

	// Start with resource constraints
	if err := sandbox.Start("", 1024, 2); err != nil { // 1GB RAM, 2 CPU cores
		log.Fatalf("Failed to start sandbox: %v", err)
	}

	// Manual cleanup without defer (for demonstration)
	var stopErr error
	defer func() {
		if stopErr != nil {
			log.Printf("Failed to stop sandbox: %v", stopErr)
		}
	}()

	// Run multiple code blocks with variable assignments
	if _, err := sandbox.Code().Run("x = 42"); err != nil {
		log.Fatalf("Failed to set x: %v", err)
	}

	if _, err := sandbox.Code().Run("y = [i**2 for i in range(10)]"); err != nil {
		log.Fatalf("Failed to set y: %v", err)
	}

	execution3, err := sandbox.Code().Run("print(f'x = {x}')\nprint(f'y = {y}')")
	if err != nil {
		log.Fatalf("Failed to run final code: %v", err)
	}

	if output, err := execution3.GetOutput(); err != nil {
		log.Printf("Failed to get output: %v", err)
	} else {
		fmt.Printf("Output: %s\n", output)
	}

	// Demonstrate error handling
	errorExecution, err := sandbox.Code().Run("1/0") // This will raise a ZeroDivisionError
	if err != nil {
		fmt.Printf("Caught error: %v\n", err)
	} else if errorOutput, err := errorExecution.GetError(); err != nil {
		log.Printf("Failed to get error output: %v", err)
	} else {
		fmt.Printf("Error: %s\n", errorOutput)
	}

	// Manual cleanup
	stopErr = sandbox.Stop()
}

// executionChainingExample demonstrates execution chaining with variables.
func executionChainingExample() {
	fmt.Println("\n=== Execution Chaining Example ===")

	sandbox := msb.NewPythonSandbox(
		msb.WithName("sandbox-chain"),
	)

	if err := sandbox.Start("", 512, 1); err != nil {
		log.Fatalf("Failed to start sandbox: %v", err)
	}
	defer func() {
		if err := sandbox.Stop(); err != nil {
			log.Printf("Failed to stop sandbox: %v", err)
		}
	}()

	// Execute a sequence of related code blocks
	if _, err := sandbox.Code().Run("name = 'Python'"); err != nil {
		log.Fatalf("Failed to set name: %v", err)
	}

	if _, err := sandbox.Code().Run("import sys"); err != nil {
		log.Fatalf("Failed to import sys: %v", err)
	}

	if _, err := sandbox.Code().Run("version = sys.version"); err != nil {
		log.Fatalf("Failed to set version: %v", err)
	}

	exec, err := sandbox.Code().Run("print(f'Hello from {name} {version}!')")
	if err != nil {
		log.Fatalf("Failed to run final code: %v", err)
	}

	// Only get output from the final execution
	if output, err := exec.GetOutput(); err != nil {
		log.Printf("Failed to get output: %v", err)
	} else {
		fmt.Printf("Output: %s\n", output)
	}
}

// dataProcessingExample demonstrates more complex data processing scenarios.
func dataProcessingExample() {
	fmt.Println("\n=== Data Processing Example ===")

	sandbox := msb.NewPythonSandbox(
		msb.WithName("sandbox-data"),
	)

	if err := sandbox.Start("", 1024, 2); err != nil {
		log.Fatalf("Failed to start sandbox: %v", err)
	}
	defer func() {
		if err := sandbox.Stop(); err != nil {
			log.Printf("Failed to stop sandbox: %v", err)
		}
	}()

	// Set up data processing environment
	setupCode := `
import json
import statistics
from collections import Counter

# Sample data
data = [
    {"name": "Alice", "age": 30, "department": "Engineering", "salary": 75000},
    {"name": "Bob", "age": 25, "department": "Marketing", "salary": 60000},
    {"name": "Charlie", "age": 35, "department": "Engineering", "salary": 85000},
    {"name": "Diana", "age": 28, "department": "Sales", "salary": 55000},
    {"name": "Eve", "age": 32, "department": "Engineering", "salary": 80000},
]

print(f"Loaded {len(data)} employee records")
`
	if _, err := sandbox.Code().Run(setupCode); err != nil {
		log.Fatalf("Failed to run setup code: %v", err)
	}

	// Process the data
	analysisCode := `
# Calculate statistics
ages = [person["age"] for person in data]
salaries = [person["salary"] for person in data]

print(f"Average age: {statistics.mean(ages):.1f}")
print(f"Average salary: ${statistics.mean(salaries):,.0f}")
print(f"Salary range: ${min(salaries):,} - ${max(salaries):,}")

# Department analysis
departments = [person["department"] for person in data]
dept_counts = Counter(departments)
print(f"\\nDepartment distribution:")
for dept, count in dept_counts.items():
    print(f"  {dept}: {count} employees")

# Find highest paid in each department
dept_salaries = {}
for person in data:
    dept = person["department"]
    if dept not in dept_salaries or person["salary"] > dept_salaries[dept]["salary"]:
        dept_salaries[dept] = person

print(f"\\nHighest paid in each department:")
for dept, person in dept_salaries.items():
    print(f"  {dept}: {person['name']} (${person['salary']:,})")
`
	analysisExecution, err := sandbox.Code().Run(analysisCode)
	if err != nil {
		log.Fatalf("Failed to run analysis code: %v", err)
	}

	if output, err := analysisExecution.GetOutput(); err != nil {
		log.Printf("Failed to get analysis output: %v", err)
	} else {
		fmt.Println(output)
	}
}

// errorRecoveryExample demonstrates error handling and recovery patterns.
func errorRecoveryExample() {
	fmt.Println("\n=== Error Recovery Example ===")

	sandbox := msb.NewPythonSandbox(
		msb.WithName("sandbox-error-recovery"),
	)

	if err := sandbox.Start("", 512, 1); err != nil {
		log.Fatalf("Failed to start sandbox: %v", err)
	}
	defer func() {
		if err := sandbox.Stop(); err != nil {
			log.Printf("Failed to stop sandbox: %v", err)
		}
	}()

	// Set up some initial state
	if _, err := sandbox.Code().Run("counter = 0"); err != nil {
		log.Fatalf("Failed to initialize counter: %v", err)
	}

	// Function that will sometimes fail
	setupCode := `
def risky_operation(value):
    global counter
    counter += 1
    if value < 0:
        raise ValueError("Negative values not allowed!")
    if value > 100:
        raise ValueError("Value too large!")
    return value * 2

print("Risky operation function defined")
`
	if _, err := sandbox.Code().Run(setupCode); err != nil {
		log.Fatalf("Failed to setup risky operation: %v", err)
	}

	// Test with valid input
	validExecution, err := sandbox.Code().Run(`
result = risky_operation(25)
print(f"Success: risky_operation(25) = {result}")
print(f"Counter is now: {counter}")
`)
	if err != nil {
		log.Printf("Failed to run valid operation: %v", err)
	} else if output, err := validExecution.GetOutput(); err != nil {
		log.Printf("Failed to get valid output: %v", err)
	} else {
		fmt.Println(output)
	}

	// Test with invalid input (should show error but sandbox continues)
	invalidExecution, err := sandbox.Code().Run(`
try:
    result = risky_operation(-5)
    print(f"Unexpected success: {result}")
except ValueError as e:
    print(f"Caught expected error: {e}")
    print(f"Counter is now: {counter}")
`)
	if err != nil {
		log.Printf("Failed to run invalid operation: %v", err)
	} else {
		if output, err := invalidExecution.GetOutput(); err != nil {
			log.Printf("Failed to get invalid output: %v", err)
		} else {
			fmt.Println(output)
		}

		if invalidExecution.HasError() {
			if errorOutput, err := invalidExecution.GetError(); err != nil {
				log.Printf("Failed to get error output: %v", err)
			} else {
				fmt.Printf("Error output: %s\n", errorOutput)
			}
		}
	}

	// Verify sandbox is still functional
	finalExecution, err := sandbox.Code().Run(`
print(f"Sandbox is still working! Final counter: {counter}")
print("Error recovery complete")
`)
	if err != nil {
		log.Printf("Failed to run final check: %v", err)
	} else if output, err := finalExecution.GetOutput(); err != nil {
		log.Printf("Failed to get final output: %v", err)
	} else {
		fmt.Println(output)
	}
}

func main() {
	fmt.Println("Python REPL Sandbox Examples")
	fmt.Println("============================")

	contextManagerEquivalentExample()
	explicitLifecycleExample()
	executionChainingExample()
	dataProcessingExample()
	errorRecoveryExample()

	fmt.Println("\nAll examples completed!")
}

