# Microsandbox Java SDK

A minimal Java SDK for the Microsandbox project.

## Installation

### Maven

```xml
<dependency>
    <groupId>dev.microsandbox</groupId>
    <artifactId>microsandbox</artifactId>
    <version>0.1.0</version>
</dependency>
```

### Gradle

```groovy
implementation 'dev.microsandbox:microsandbox:0.1.0'
```

## Usage

```java
import dev.microsandbox.HelloWorld;

public class Example {
    public static void main(String[] args) {
        // Print a greeting
        HelloWorld.greet("World");
    }
}
```

## Examples

Check out the `src/main/java/dev/microsandbox/examples` directory for examples of how to use this SDK.

You can run the examples with Maven:

```bash
# Build the project
mvn clean install

# Run the example
mvn exec:java
```

## Development

See [DEVELOPMENT.md](DEVELOPMENT.md) for information on building from source and contributing to the project.

## License

[Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0)
