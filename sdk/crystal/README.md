# Microsandbox Crystal SDK

A lightweight Crystal SDK for interacting with the Microsandbox service.

## Installation

1. Add the dependency to your `shard.yml`:

```yaml
dependencies:
  microsandbox:
    github: microsandbox/crystal
```

2. Run `shards install`

## Usage

```crystal
require "microsandbox"

# Simple greeting
puts Microsandbox.greet("World")
```

## License

[Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0)
