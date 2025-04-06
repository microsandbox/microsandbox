require "./microsandbox/version"

# Main module for the Microsandbox library
module Microsandbox
  # Returns a greeting message with the given name
  #
  # ## Example
  #
  # ```
  # Microsandbox.greet("World") # => "Hello, World!"
  # ```
  def self.greet(name : String) : String
    "Hello, #{name}!"
  end
end
