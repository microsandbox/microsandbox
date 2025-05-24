---
order: 60
icon: project
tags: [guide]
---

# Projects

Learn how to use microsandbox's project-based development workflow for managing complex sandbox environments and persistent development setups.

!!!info What You'll Learn

- How to create and manage sandbox projects
- Working with Sandboxfiles
- Understanding different persistence models
- Project workflow best practices
  !!!

### Overview

microsandbox supports project-based development similar to npm, cargo, or other package managers. This approach is perfect for:

- **Development environments** that need to persist between sessions
- **Complex applications** with multiple services or components
- **Team collaboration** with shared sandbox configurations
- **Reproducible environments** across different machines

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

#### View Your Sandboxfile

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
# or use the shorthand:
msr myapp
```

This executes the default _start_ script of your sandbox. For more control, you can specify which script to run:

```bash
msr myapp~start
```

### Sandbox Types and Persistence

microsandbox provides different sandbox types optimized for various use cases:

=== :icon-code: **Sandbox Types**

**PythonSandbox**

- Optimized for Python code execution
- Pre-installed Python runtime and common tools
- Fast startup for Python workloads

**NodeSandbox**

- Optimized for JavaScript/Node.js code
- Pre-installed Node.js runtime and npm
- Perfect for web development and JavaScript applications

**GenericSandbox**

- For custom container images
- Maximum flexibility for any runtime or environment
- Use any Docker-compatible image

=== :icon-database: **Persistence Models**

**Temporary Sandboxes**

- Clean environment with no persistence
- Perfect for one-off tasks and experimentation
- All changes discarded when sandbox stops

**Project Sandboxes**

- Persistent state in `./menv` directory
- File changes and installations are saved
- Resume exactly where you left off

**Installed Sandboxes**

- System-wide persistence
- Available from anywhere on your system
- # Managed through `msb install` command

### Working with Project Sandboxes

#### Persistence Directory

When running project sandboxes, all file changes and installations are automatically persisted to the `./menv` directory in your project root:

```
my-project/
├── Sandboxfile
├── menv/           # Persistent sandbox data
│   └── myapp/      # Data for 'myapp' sandbox
├── src/
└── README.md
```

This means you can:

- Stop and restart sandboxes without losing work
- Install packages that persist between sessions
- Create files that remain available
- Maintain development state across restarts

#### Multiple Sandboxes

You can define multiple sandboxes in a single project:

```bash
# Add a web frontend
msb add frontend \
    --image node \
    --cpus 2 \
    --memory 2048 \
    --start 'npm run dev'

# Add a database
msb add database \
    --image postgres \
    --memory 1024 \
    --start 'postgres'

# Add a Python API
msb add api \
    --image python \
    --memory 1024 \
    --start 'python -m uvicorn main:app --reload'
```

Your `Sandboxfile` will contain all configurations:

```yaml
sandboxes:
  frontend:
    image: node
    memory: 2048
    cpus: 2
    scripts:
      start: npm run dev

  database:
    image: postgres
    memory: 1024
    scripts:
      start: postgres

  api:
    image: python
    memory: 1024
    scripts:
      start: python -m uvicorn main:app --reload
```

#### Custom Scripts

Define custom scripts for different tasks:

```bash
msb add myapp \
    --image python \
    --start 'python app.py' \
    --script test='python -m pytest' \
    --script lint='flake8 .' \
    --script build='python setup.py build'
```

Run specific scripts:

```bash
msr myapp~test    # Run tests
msr myapp~lint    # Run linter
msr myapp~build   # Build project
```

### Temporary Sandboxes

For experimentation or one-off tasks, use temporary sandboxes:

```bash
msb exe --image python
# or the shorthand:
msx python
```

Temporary sandboxes:

- Provide a clean environment
- Leave no trace when finished
- Perfect for testing untrusted code
- Ideal for quick experiments

### Installing Sandboxes

Install a sandbox as a system-wide executable:

```bash
msb install --image alpine
# or the shorthand:
msi alpine
```

After installation, start your sandbox from anywhere:

```bash
alpine
```

#### Named Installations

Give your sandbox a descriptive name:

```bash
msi python:3.11 python-data-science
msi node:18 node-web-dev
```

This creates multiple instances with different configurations:

- `python-data-science` - Python environment for data analysis
- `node-web-dev` - Node.js environment for web development

### Best Practices

#### Project Organization

Structure your projects for clarity:

```
my-project/
├── Sandboxfile          # Sandbox configurations
├── menv/               # Persistent data (auto-generated)
├── src/                # Source code
├── docs/               # Documentation
├── tests/              # Test files
└── README.md           # Project documentation
```

#### Naming Conventions

Use descriptive names for sandboxes:

```yaml
sandboxes:
  web-frontend: # Clear purpose
    image: node

  python-api: # Technology + role
    image: python

  postgres-db: # Service type
    image: postgres
```

#### Resource Management

Set appropriate resource limits:

```yaml
sandboxes:
  lightweight-task:
    image: alpine
    memory: 512 # Minimal resources
    cpus: 1

  data-processing:
    image: python
    memory: 4096 # More memory for data work
    cpus: 4
```

#### Script Organization

Define scripts for common tasks:

```yaml
sandboxes:
  myapp:
    image: python
    scripts:
      start: python app.py
      dev: python app.py --debug
      test: python -m pytest
      lint: flake8 .
      format: black .
      install: pip install -r requirements.txt
```

### Advanced Configuration

#### Environment Variables

Set environment variables for your sandboxes:

```yaml
sandboxes:
  myapp:
    image: python
    environment:
      DEBUG: "true"
      DATABASE_URL: "postgresql://localhost/mydb"
      API_KEY: "${API_KEY}" # Use host environment variable
```

#### Port Mapping

Expose ports for web applications:

```yaml
sandboxes:
  webapp:
    image: node
    ports:
      - "3000:3000" # Map host:container
      - "8080:8080"
    scripts:
      start: npm run dev
```

#### Volume Mounts

Mount host directories:

```yaml
sandboxes:
  development:
    image: python
    volumes:
      - "./src:/app/src" # Mount source code
      - "./data:/app/data:ro" # Read-only data mount
```

### Troubleshooting

#### Sandbox Won't Start

Check your Sandboxfile syntax:

```bash
msb validate  # Validate Sandboxfile
```

#### Persistence Issues

If changes aren't persisting, check the `menv` directory permissions:

```bash
ls -la menv/
```

#### Resource Conflicts

If sandboxes conflict over resources, adjust memory/CPU limits:

```yaml
sandboxes:
  resource-heavy:
    memory: 2048 # Reduce if needed
    cpus: 2 # Limit CPU usage
```

### Next Steps

Now that you understand projects, explore:

- [!ref Architecture Guide](architecture.md) - How microsandbox works internally
- [!ref Configuration Reference](/references/configuration.md) - Complete configuration options
- [!ref CLI Reference](/references/cli.md) - All available commands
- [!ref SDK Documentation](/sdk/) - Programmatic sandbox management
