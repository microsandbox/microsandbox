# Microsandbox Swift SDK

A minimal Swift SDK for the Microsandbox project.

## Installation

### Swift Package Manager

Add the following to your `Package.swift` file:

```swift
dependencies: [
    .package(url: "https://github.com/yourusername/monocore.git", from: "0.0.1")
]
```

Then specify the "Microsandbox" product as a dependency for your target:

```swift
targets: [
    .target(
        name: "YourTarget",
        dependencies: [
            .product(name: "Microsandbox", package: "monocore")
        ]
    )
]
```

### CocoaPods

Add the following to your Podfile:

```ruby
pod 'Microsandbox', '~> 0.1.0'
```

Then run:

```bash
pod install
```

## Usage

```swift
import Microsandbox

// Print a greeting
Microsandbox.greet("World")
```

## License

[Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0)
