# frozen_string_literal: true

require_relative "microsandbox/version"

# Microsandbox SDK for Ruby
module Microsandbox
  class Error < StandardError; end

  # Returns a greeting message for the given name
  #
  # @param name [String] The name to greet
  # @return [String] A greeting message
  def self.greet(name)
    "Hello, #{name}!"
  end
end
