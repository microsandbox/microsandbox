package main

import (
	"fmt"
	"log"

	msb "github.com/keithang/microsandbox/sdk/go"
)

// basicExample demonstrates basic JavaScript code execution.
func basicExample() {
	fmt.Println("\n=== Basic Node.js Example ===")

	// Create a Node.js sandbox
	sandbox := msb.NewNodeSandbox(
		msb.WithName("node-basic"),
	)

	if err := sandbox.Start("", 512, 1); err != nil {
		log.Fatalf("Failed to start sandbox: %v", err)
	}
	defer func() {
		if err := sandbox.Stop(); err != nil {
			log.Printf("Failed to stop sandbox: %v", err)
		}
	}()

	// Run a simple JavaScript code snippet
	execution, err := sandbox.Code().Run("console.log('Hello from Node.js!');")
	if err != nil {
		log.Fatalf("Failed to run code: %v", err)
	}

	if output, err := execution.GetOutput(); err != nil {
		log.Printf("Failed to get output: %v", err)
	} else {
		fmt.Printf("Output: %s\n", output)
	}

	// Run JavaScript code that uses Node.js functionality
	versionCode := `
const version = process.version;
const platform = process.platform;
console.log(` + "`" + `Node.js ${version} running on ${platform}` + "`" + `);`
	versionExecution, err := sandbox.Code().Run(versionCode)
	if err != nil {
		log.Fatalf("Failed to run version code: %v", err)
	}

	if output, err := versionExecution.GetOutput(); err != nil {
		log.Printf("Failed to get version output: %v", err)
	} else {
		fmt.Printf("Node.js info: %s\n", output)
	}
}

// errorHandlingExample demonstrates how to handle JavaScript errors.
func errorHandlingExample() {
	fmt.Println("\n=== Error Handling Example ===")

	sandbox := msb.NewNodeSandbox(
		msb.WithName("node-error"),
	)

	if err := sandbox.Start("", 512, 1); err != nil {
		log.Fatalf("Failed to start sandbox: %v", err)
	}
	defer func() {
		if err := sandbox.Stop(); err != nil {
			log.Printf("Failed to stop sandbox: %v", err)
		}
	}()

	// Run code with a caught error
	caughtErrorCode := `
try {
    // This will cause a ReferenceError
    console.log(undefinedVariable);
} catch (error) {
    console.error('Caught error:', error.message);
}
`
	caughtExecution, err := sandbox.Code().Run(caughtErrorCode)
	if err != nil {
		log.Fatalf("Failed to run caught error code: %v", err)
	}

	if output, err := caughtExecution.GetOutput(); err != nil {
		log.Printf("Failed to get standard output: %v", err)
	} else {
		fmt.Printf("Standard output: %s\n", output)
	}

	if errorOutput, err := caughtExecution.GetError(); err != nil {
		log.Printf("Failed to get error output: %v", err)
	} else {
		fmt.Printf("Error output: %s\n", errorOutput)
	}

	fmt.Printf("Has error: %t\n", caughtExecution.HasError())
}

// moduleExample demonstrates Node.js module usage.
func moduleExample() {
	fmt.Println("\n=== Module Usage Example ===")

	sandbox := msb.NewNodeSandbox(
		msb.WithName("node-module"),
	)

	if err := sandbox.Start("", 512, 1); err != nil {
		log.Fatalf("Failed to start sandbox: %v", err)
	}
	defer func() {
		if err := sandbox.Stop(); err != nil {
			log.Printf("Failed to stop sandbox: %v", err)
		}
	}()

	// Using built-in Node.js modules
	fsCode := `
const fs = require('fs');
const os = require('os');

// Write a file
fs.writeFileSync('/tmp/hello.txt', 'Hello from Node.js!');
console.log('File written successfully');

// Read the file back
const content = fs.readFileSync('/tmp/hello.txt', 'utf8');
console.log('File content:', content);

// Get system info
console.log('Hostname:', os.hostname());
console.log('Platform:', os.platform());
console.log('Architecture:', os.arch());
`
	fsExecution, err := sandbox.Code().Run(fsCode)
	if err != nil {
		log.Fatalf("Failed to run fs code: %v", err)
	}

	if output, err := fsExecution.GetOutput(); err != nil {
		log.Printf("Failed to get fs output: %v", err)
	} else {
		fmt.Println(output)
	}
}

// executionChainingExample demonstrates execution chaining with variables.
func executionChainingExample() {
	fmt.Println("\n=== Execution Chaining Example ===")

	sandbox := msb.NewNodeSandbox(
		msb.WithName("node-chain"),
	)

	if err := sandbox.Start("", 512, 1); err != nil {
		log.Fatalf("Failed to start sandbox: %v", err)
	}
	defer func() {
		if err := sandbox.Stop(); err != nil {
			log.Printf("Failed to stop sandbox: %v", err)
		}
	}()

	// Execute a sequence of related code blocks that maintain state
	if _, err := sandbox.Code().Run("const name = 'Node.js';"); err != nil {
		log.Fatalf("Failed to set name variable: %v", err)
	}

	if _, err := sandbox.Code().Run("const version = process.version;"); err != nil {
		log.Fatalf("Failed to set version variable: %v", err)
	}

	if _, err := sandbox.Code().Run("const numbers = [1, 2, 3, 4, 5];"); err != nil {
		log.Fatalf("Failed to set numbers variable: %v", err)
	}

	// Use variables from previous executions
	finalExecution, err := sandbox.Code().Run(`
	console.log(` + "`" + `Hello from ${name} ${version}!` + "`" + `);
const sum = numbers.reduce((a, b) => a + b, 0);
console.log(` + "`" + `Sum of numbers: ${sum}` + "`" + `);
`)
	if err != nil {
		log.Fatalf("Failed to run final code: %v", err)
	}

	if output, err := finalExecution.GetOutput(); err != nil {
		log.Printf("Failed to get final output: %v", err)
	} else {
		fmt.Println(output)
	}
}

// jsonAndDataExample demonstrates working with JSON and data structures.
func jsonAndDataExample() {
	fmt.Println("\n=== JSON and Data Example ===")

	sandbox := msb.NewNodeSandbox(
		msb.WithName("node-json"),
	)

	if err := sandbox.Start("", 512, 1); err != nil {
		log.Fatalf("Failed to start sandbox: %v", err)
	}
	defer func() {
		if err := sandbox.Stop(); err != nil {
			log.Printf("Failed to stop sandbox: %v", err)
		}
	}()

	// Working with JSON data
	jsonCode := `
// Create some sample data
const data = {
    users: [
        { id: 1, name: 'Alice', email: 'alice@example.com' },
        { id: 2, name: 'Bob', email: 'bob@example.com' },
        { id: 3, name: 'Charlie', email: 'charlie@example.com' }
    ],
    metadata: {
        total: 3,
        created: new Date().toISOString()
    }
};

// Convert to JSON and back
const jsonString = JSON.stringify(data, null, 2);
console.log('JSON representation:');
console.log(jsonString);

// Parse and manipulate
const parsed = JSON.parse(jsonString);
const userNames = parsed.users.map(user => user.name);
console.log('\\nUser names:', userNames.join(', '));

// Filter and transform
const emailDomains = parsed.users
    .map(user => user.email.split('@')[1])
    .filter((domain, index, arr) => arr.indexOf(domain) === index);
console.log('Unique email domains:', emailDomains);
`
	jsonExecution, err := sandbox.Code().Run(jsonCode)
	if err != nil {
		log.Fatalf("Failed to run JSON code: %v", err)
	}

	if output, err := jsonExecution.GetOutput(); err != nil {
		log.Printf("Failed to get JSON output: %v", err)
	} else {
		fmt.Println(output)
	}
}

func main() {
	fmt.Println("Node.js Sandbox Examples")
	fmt.Println("=======================")

	basicExample()
	errorHandlingExample()
	moduleExample()
	executionChainingExample()
	jsonAndDataExample()

	fmt.Println("\nAll Node.js examples completed!")
}
