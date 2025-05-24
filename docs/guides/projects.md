---
order: 60
icon: project
tags: [guide]
---

# Projects

Learn how to use microsandbox's project-based development workflow for managing complex sandbox environments and persistent development setups.

---

### Overview

microsandbox supports project-based development similar to npm, cargo, or other package managers. This approach is perfect for:

- **Development environments** that need to persist between sessions
- **Complex applications** with multiple services or components
- **Team collaboration** with shared sandbox configurations
- **Reproducible environments** across different machines

---

### Project-Based Development

#### Initialize a Project

Create a new microsandbox project in your current directory:

```bash
msb init
```

This creates a `Sandboxfile` in your current directory, which serves as the configuration manifest for your sandbox environments.

#### Add a Sandbox to Your Project

Register a new sandbox in your project:

```bash
msb add myapp \
    --image python \
    --cpus 1 \
    --memory 1024 \
    --start 'python app.py'
```

This command adds a sandbox named `myapp` to your Sandboxfile with the specified configuration.

After adding a sandbox, your `Sandboxfile` will look like this:

```yaml
# Sandbox configurations
sandboxes:
  myapp:
    image: python
    memory: 1024
    cpus: 1
    scripts:
      start: python app.py
```

#### Run Your Project Sandbox

Execute your project sandbox:

```bash
msb run --sandbox myapp
```

or use the shorthand:

```bash
msr myapp
```

This executes the default _start_ script of your sandbox. For more control, you can specify which script to run:

```bash
msr myapp~start
```
