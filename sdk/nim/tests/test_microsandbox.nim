import unittest
import microsandbox
import strutils  # Add this import for the contains procedure

test "greet returns correct greeting":
  let result = greet("World")
  check("Hello, World! Welcome to Microsandbox!" == result)

test "greet includes the provided name":
  let name = "Tester"
  let result = greet(name)
  check(result.contains(name))  # Use contains procedure instead of 'in' operator
