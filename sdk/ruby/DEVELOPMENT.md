## Development

### Setup

```bash
# Clone the repository
git clone https://github.com/yourusername/monocore.git
cd monocore/sdk/ruby

# Install a compatible version of bundler
gem install bundler -v 2.4.22

# Install dependencies
bundle _2.4.22_ install
```

### Running Tests

```bash
# Run all tests
rake test

# Run a specific test file
ruby -Ilib:test test/microsandbox_test.rb
```

### Building the Gem

```bash
gem build microsandbox.gemspec
```

### Publishing to RubyGems

```bash
# Login to RubyGems (if not already logged in)
gem signin

# Publish the gem
gem push microsandbox-0.1.0.gem
```

Make sure you have registered for an account on [RubyGems](https://rubygems.org/) and verified your email address.
