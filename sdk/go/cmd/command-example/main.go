package main

import (
	"fmt"
	"log"

	msb "github.com/keithang/microsandbox/sdk/go"
)

// basicExample demonstrates basic command execution with proper lifecycle management.
func basicExample() {
	fmt.Println("\n=== Basic Command Example ===")

	// Create a sandbox with explicit configuration
	sandbox := msb.NewPythonSandbox(
		msb.WithName("command-example"),
	)

	// Start the sandbox
	if err := sandbox.Start("", 512, 1); err != nil {
		log.Fatalf("Failed to start sandbox: %v", err)
	}
	defer func() {
		if err := sandbox.Stop(); err != nil {
			log.Printf("Failed to stop sandbox: %v", err)
		}
	}()

	// Run a simple command
	lsExecution, err := sandbox.Command().Run("ls", []string{"-la", "/"})
	if err != nil {
		log.Fatalf("Failed to run ls command: %v", err)
	}

	fmt.Println("$ ls -la /")
	fmt.Printf("Exit code: %d\n", lsExecution.GetExitCode())
	fmt.Println("Output:")
	if output, err := lsExecution.GetOutput(); err != nil {
		log.Printf("Failed to get output: %v", err)
	} else {
		fmt.Println(output)
	}

	// Execute a command with string arguments
	echoExecution, err := sandbox.Command().Run("echo", []string{"Hello from", "sandbox command!"})
	if err != nil {
		log.Fatalf("Failed to run echo command: %v", err)
	}

	fmt.Println("\n$ echo Hello from sandbox command!")
	if output, err := echoExecution.GetOutput(); err != nil {
		log.Printf("Failed to get output: %v", err)
	} else {
		fmt.Printf("Output: %s\n", output)
	}

	// Get system information
	unameExecution, err := sandbox.Command().Run("uname", []string{"-a"})
	if err != nil {
		log.Fatalf("Failed to run uname command: %v", err)
	}

	fmt.Println("\n$ uname -a")
	if output, err := unameExecution.GetOutput(); err != nil {
		log.Printf("Failed to get output: %v", err)
	} else {
		fmt.Printf("Output: %s\n", output)
	}
}

// errorHandlingExample demonstrates how to handle command errors.
func errorHandlingExample() {
	fmt.Println("\n=== Error Handling Example ===")

	sandbox := msb.NewPythonSandbox(
		msb.WithName("error-example"),
	)

	if err := sandbox.Start("", 512, 1); err != nil {
		log.Fatalf("Failed to start sandbox: %v", err)
	}
	defer func() {
		if err := sandbox.Stop(); err != nil {
			log.Printf("Failed to stop sandbox: %v", err)
		}
	}()

	// Run a command that generates an error
	errorExecution, err := sandbox.Command().Run("ls", []string{"/nonexistent"})
	if err != nil {
		log.Printf("Command execution failed: %v", err)
		return
	}

	fmt.Println("$ ls /nonexistent")
	fmt.Printf("Exit code: %d\n", errorExecution.GetExitCode())
	fmt.Printf("Success: %t\n", errorExecution.IsSuccess())
	fmt.Println("Error output:")
	if errorOutput, err := errorExecution.GetError(); err != nil {
		log.Printf("Failed to get error output: %v", err)
	} else {
		fmt.Println(errorOutput)
	}

	// Deliberately cause a command not found error
	_, err = sandbox.Command().Run("nonexistentcommand", []string{})
	if err != nil {
		fmt.Printf("\nCaught error for nonexistent command: %v\n", err)
	}
}

// advancedExample demonstrates more complex command usage patterns.
func advancedExample() {
	fmt.Println("\n=== Advanced Example ===")

	sandbox := msb.NewPythonSandbox(
		msb.WithName("advanced-example"),
	)

	if err := sandbox.Start("", 1024, 2); err != nil {
		log.Fatalf("Failed to start sandbox: %v", err)
	}
	defer func() {
		if err := sandbox.Stop(); err != nil {
			log.Printf("Failed to stop sandbox: %v", err)
		}
	}()

	// Write a file
	writeCmd, err := sandbox.Command().Run("bash", []string{"-c", "echo 'Hello, file content!' > /tmp/test.txt"})
	if err != nil {
		log.Fatalf("Failed to write file: %v", err)
	}
	fmt.Printf("Created file, exit code: %d\n", writeCmd.GetExitCode())

	// Read the file back
	readCmd, err := sandbox.Command().Run("cat", []string{"/tmp/test.txt"})
	if err != nil {
		log.Fatalf("Failed to read file: %v", err)
	}
	if content, err := readCmd.GetOutput(); err != nil {
		log.Printf("Failed to get file content: %v", err)
	} else {
		fmt.Printf("File content: %s\n", content)
	}

	// Run a more complex pipeline
	pipelineCmd, err := sandbox.Command().Run("bash", []string{
		"-c",
		"mkdir -p /tmp/test_dir && " +
			"echo 'Line 1' > /tmp/test_dir/data.txt && " +
			"echo 'Line 2' >> /tmp/test_dir/data.txt && " +
			"cat /tmp/test_dir/data.txt | grep 'Line' | wc -l",
	})
	if err != nil {
		log.Fatalf("Failed to run pipeline: %v", err)
	}

	if output, err := pipelineCmd.GetOutput(); err != nil {
		log.Printf("Failed to get pipeline output: %v", err)
	} else {
		fmt.Printf("\nPipeline output (should be 2): %s\n", output)
	}

	// Create and run a Python script
	createScript, err := sandbox.Command().Run("bash", []string{
		"-c",
		`cat > /tmp/test.py << 'EOF'
import sys
print("Python script executed!")
print(f"Arguments: {sys.argv[1:]}")
EOF`,
	})
	if err != nil {
		log.Fatalf("Failed to create script: %v", err)
	}

	if createScript.IsSuccess() {
		// Run the script with arguments
		scriptCmd, err := sandbox.Command().Run("python", []string{"/tmp/test.py", "arg1", "arg2", "arg3"})
		if err != nil {
			log.Fatalf("Failed to run script: %v", err)
		}

		fmt.Println("\nPython script output:")
		if output, err := scriptCmd.GetOutput(); err != nil {
			log.Printf("Failed to get script output: %v", err)
		} else {
			fmt.Println(output)
		}
	}
}

// explicitLifecycleExample demonstrates explicit lifecycle management without defer.
func explicitLifecycleExample() {
	fmt.Println("\n=== Explicit Lifecycle Example ===")

	// Create sandbox with custom server URL
	sandbox := msb.NewPythonSandbox(
		msb.WithName("explicit-lifecycle"),
		msb.WithServerUrl("http://127.0.0.1:5555"),
	)

	// Manually start the sandbox
	fmt.Println("Starting sandbox...")
	if err := sandbox.Start("", 512, 1); err != nil {
		log.Fatalf("Failed to start sandbox: %v", err)
	}

	// Execute commands
	hostnameCmd, err := sandbox.Command().Run("hostname", []string{})
	if err != nil {
		log.Printf("Failed to get hostname: %v", err)
	} else if output, err := hostnameCmd.GetOutput(); err != nil {
		log.Printf("Failed to get hostname output: %v", err)
	} else {
		fmt.Printf("Hostname: %s\n", output)
	}

	dateCmd, err := sandbox.Command().Run("date", []string{})
	if err != nil {
		log.Printf("Failed to get date: %v", err)
	} else if output, err := dateCmd.GetOutput(); err != nil {
		log.Printf("Failed to get date output: %v", err)
	} else {
		fmt.Printf("Date: %s\n", output)
	}

	// Manually stop the sandbox
	fmt.Println("Stopping sandbox...")
	if err := sandbox.Stop(); err != nil {
		log.Printf("Failed to stop sandbox: %v", err)
	}
}

func main() {
	fmt.Println("Command Execution Examples")
	fmt.Println("=========================")

	basicExample()
	errorHandlingExample()
	advancedExample()
	explicitLifecycleExample()

	fmt.Println("\nAll examples completed!")
}
