---
icon: code
title: Examples
description: Practical examples and templates for common use cases
order: 500
---

# Examples

Practical examples and templates to help you get started with Microsandbox. These examples demonstrate common patterns and best practices for different development scenarios.

## Quick Examples

### Simple Python Application

```bash
# Create and run a Python sandbox
msb exe python -- python -c "print('Hello from Microsandbox!')"

# Run with file mounting
msb exe python \
    --volume "./app.py:/workspace/app.py" \
    -- python /workspace/app.py
```

### Web Development Server

```bash
# Node.js development server
msb exe node \
    --port "3000:3000" \
    --volume "./src:/workspace" \
    --workdir "/workspace" \
    -- npm run dev

# Python Flask server
msb exe python \
    --port "5000:5000" \
    --volume "./app:/workspace" \
    --env "FLASK_ENV=development" \
    -- python app.py
```

### Database Operations

```bash
# PostgreSQL database
msb exe postgres \
    --port "5432:5432" \
    --env "POSTGRES_DB=myapp" \
    --env "POSTGRES_USER=user" \
    --env "POSTGRES_PASSWORD=password" \
    --volume "./data:/var/lib/postgresql/data"

# Connect to database
msb exe postgres \
    --link "db:postgres" \
    -- psql -h db -U user myapp
```

## Project Examples

### Full-Stack Web Application

Create a `Sandboxfile` for a complete web application:

```yaml
# Sandboxfile
sandboxes:
  frontend:
    image: node:18
    ports: ["3000:3000"]
    volumes: ["./frontend:/workspace"]
    workdir: "/workspace"
    scripts:
      start: npm run dev
      build: npm run build
      test: npm test
    environment:
      REACT_APP_API_URL: "http://localhost:8000"

  backend:
    image: python:3.11
    ports: ["8000:8000"]
    volumes: ["./backend:/workspace"]
    workdir: "/workspace"
    environment:
      DATABASE_URL: "postgresql://user:password@db:5432/myapp"
      REDIS_URL: "redis://cache:6379"
    scripts:
      start: uvicorn main:app --host 0.0.0.0 --port 8000 --reload
      test: pytest tests/
      migrate: alembic upgrade head
    depends_on: [db, cache]

  db:
    image: postgres:15
    environment:
      POSTGRES_DB: myapp
      POSTGRES_USER: user
      POSTGRES_PASSWORD: password
    volumes: ["./data/postgres:/var/lib/postgresql/data"]
    ports: ["5432:5432"]

  cache:
    image: redis:7
    volumes: ["./data/redis:/data"]
    ports: ["6379:6379"]
```

Commands:

```bash
# Initialize project
msb init

# Add all sandboxes
msb add frontend --image node:18 --port "3000:3000" --volume "./frontend:/workspace"
msb add backend --image python:3.11 --port "8000:8000" --volume "./backend:/workspace" --depends-on db,cache
msb add db --image postgres:15 --env "POSTGRES_DB=myapp" --volume "./data/postgres:/var/lib/postgresql/data"
msb add cache --image redis:7 --volume "./data/redis:/data"

# Start all services
msb up

# Run specific services
msb run frontend~start
msb run backend~migrate
msb run backend~start
```

### Data Science Workflow

```yaml
# Sandboxfile for data science project
sandboxes:
  jupyter:
    image: jupyter/datascience-notebook
    ports: ["8888:8888"]
    volumes:
      - "./notebooks:/home/jovyan/work"
      - "./data:/home/jovyan/data"
      - "./models:/home/jovyan/models"
    environment:
      JUPYTER_ENABLE_LAB: "yes"
    scripts:
      start: jupyter lab --ip=0.0.0.0 --allow-root --no-browser

  processing:
    image: python:3.11
    memory: 4096
    volumes:
      - "./scripts:/workspace"
      - "./data:/data"
      - "./models:/models"
    scripts:
      preprocess: python preprocess_data.py
      train: python train_model.py
      evaluate: python evaluate_model.py
      serve: python serve_model.py

  database:
    image: postgres:15
    environment:
      POSTGRES_DB: analytics
      POSTGRES_USER: analyst
      POSTGRES_PASSWORD: password
    volumes: ["./data/postgres:/var/lib/postgresql/data"]
    ports: ["5432:5432"]
```

### Microservices Architecture

```yaml
# Sandboxfile for microservices
sandboxes:
  api-gateway:
    image: nginx
    ports: ["80:80"]
    volumes: ["./nginx.conf:/etc/nginx/nginx.conf"]
    depends_on: [user-service, order-service, product-service]

  user-service:
    image: node:18
    ports: ["3001:3000"]
    volumes: ["./services/user:/workspace"]
    environment:
      DATABASE_URL: "postgresql://user:password@user-db:5432/users"
      REDIS_URL: "redis://cache:6379"
    scripts:
      start: npm run start
      dev: npm run dev
      test: npm test
    depends_on: [user-db, cache]

  order-service:
    image: python:3.11
    ports: ["3002:8000"]
    volumes: ["./services/order:/workspace"]
    environment:
      DATABASE_URL: "postgresql://user:password@order-db:5432/orders"
    scripts:
      start: uvicorn main:app --host 0.0.0.0 --port 8000
      test: pytest
    depends_on: [order-db]

  product-service:
    image: golang:1.21
    ports: ["3003:8080"]
    volumes: ["./services/product:/workspace"]
    environment:
      DATABASE_URL: "postgresql://user:password@product-db:5432/products"
    scripts:
      start: go run main.go
      build: go build -o app
      test: go test ./...
    depends_on: [product-db]

  user-db:
    image: postgres:15
    environment:
      POSTGRES_DB: users
      POSTGRES_USER: user
      POSTGRES_PASSWORD: password
    volumes: ["./data/user-db:/var/lib/postgresql/data"]

  order-db:
    image: postgres:15
    environment:
      POSTGRES_DB: orders
      POSTGRES_USER: user
      POSTGRES_PASSWORD: password
    volumes: ["./data/order-db:/var/lib/postgresql/data"]

  product-db:
    image: postgres:15
    environment:
      POSTGRES_DB: products
      POSTGRES_USER: user
      POSTGRES_PASSWORD: password
    volumes: ["./data/product-db:/var/lib/postgresql/data"]

  cache:
    image: redis:7
    volumes: ["./data/redis:/data"]
```

## Language-Specific Examples

### Python Development

```bash
# Django project
msb exe python \
    --volume "./myproject:/workspace" \
    --port "8000:8000" \
    --env "DJANGO_SETTINGS_MODULE=myproject.settings" \
    -- python manage.py runserver 0.0.0.0:8000

# FastAPI with auto-reload
msb exe python \
    --volume "./app:/workspace" \
    --port "8000:8000" \
    -- uvicorn main:app --host 0.0.0.0 --port 8000 --reload

# Jupyter notebook
msb exe jupyter/scipy-notebook \
    --port "8888:8888" \
    --volume "./notebooks:/home/jovyan/work" \
    -- start-notebook.sh --NotebookApp.token=''
```

### Node.js Development

```bash
# Express server
msb exe node \
    --volume "./app:/workspace" \
    --port "3000:3000" \
    --workdir "/workspace" \
    -- npm run dev

# React development
msb exe node \
    --volume "./frontend:/workspace" \
    --port "3000:3000" \
    --env "CHOKIDAR_USEPOLLING=true" \
    -- npm start

# Next.js application
msb exe node \
    --volume "./nextapp:/workspace" \
    --port "3000:3000" \
    -- npm run dev
```

### Go Development

```bash
# Go web server
msb exe golang \
    --volume "./app:/workspace" \
    --port "8080:8080" \
    --workdir "/workspace" \
    -- go run main.go

# Go with hot reload
msb exe golang \
    --volume "./app:/workspace" \
    --port "8080:8080" \
    -- sh -c "go install github.com/cosmtrek/air@latest && air"
```

### Rust Development

```bash
# Rust web server
msb exe rust \
    --volume "./app:/workspace" \
    --port "8000:8000" \
    --workdir "/workspace" \
    -- cargo run

# Rust with watch mode
msb exe rust \
    --volume "./app:/workspace" \
    --port "8000:8000" \
    -- sh -c "cargo install cargo-watch && cargo watch -x run"
```

## Database Examples

### PostgreSQL

```bash
# PostgreSQL server
msb exe postgres \
    --port "5432:5432" \
    --env "POSTGRES_DB=myapp" \
    --env "POSTGRES_USER=user" \
    --env "POSTGRES_PASSWORD=password" \
    --volume "./data:/var/lib/postgresql/data"

# PostgreSQL client
msb exe postgres \
    -- psql -h host.docker.internal -p 5432 -U user -d myapp
```

### MySQL

```bash
# MySQL server
msb exe mysql \
    --port "3306:3306" \
    --env "MYSQL_ROOT_PASSWORD=rootpassword" \
    --env "MYSQL_DATABASE=myapp" \
    --env "MYSQL_USER=user" \
    --env "MYSQL_PASSWORD=password" \
    --volume "./data:/var/lib/mysql"

# MySQL client
msb exe mysql \
    -- mysql -h host.docker.internal -P 3306 -u user -p myapp
```

### MongoDB

```bash
# MongoDB server
msb exe mongo \
    --port "27017:27017" \
    --env "MONGO_INITDB_ROOT_USERNAME=admin" \
    --env "MONGO_INITDB_ROOT_PASSWORD=password" \
    --volume "./data:/data/db"

# MongoDB client
msb exe mongo \
    -- mongosh --host host.docker.internal --port 27017
```

### Redis

```bash
# Redis server
msb exe redis \
    --port "6379:6379" \
    --volume "./data:/data"

# Redis client
msb exe redis \
    -- redis-cli -h host.docker.internal -p 6379
```

## Development Workflows

### Testing Workflow

```bash
# Run tests in isolated environment
msb exe python \
    --volume "./app:/workspace" \
    --workdir "/workspace" \
    -- pytest tests/ -v

# Run tests with coverage
msb exe python \
    --volume "./app:/workspace" \
    -- sh -c "pip install pytest-cov && pytest --cov=src tests/"

# Run linting
msb exe python \
    --volume "./app:/workspace" \
    -- sh -c "pip install flake8 && flake8 src/"
```

### CI/CD Simulation

```bash
# Simulate CI pipeline
msb exe node \
    --volume "./app:/workspace" \
    --workdir "/workspace" \
    -- sh -c "
        npm ci &&
        npm run lint &&
        npm run test &&
        npm run build
    "

# Multi-stage build simulation
msb exe python \
    --volume "./app:/workspace" \
    -- sh -c "
        pip install -r requirements-dev.txt &&
        pytest tests/ &&
        pip install -r requirements.txt &&
        python -m build
    "
```

### Environment Isolation

```bash
# Test with different Python versions
msb exe python:3.9 --volume "./app:/workspace" -- python --version
msb exe python:3.10 --volume "./app:/workspace" -- python --version
msb exe python:3.11 --volume "./app:/workspace" -- python --version

# Test with different Node versions
msb exe node:16 --volume "./app:/workspace" -- node --version
msb exe node:18 --volume "./app:/workspace" -- node --version
msb exe node:20 --volume "./app:/workspace" -- node --version
```

## Networking Examples

### Service Communication

```bash
# Start database
msb exe postgres \
    --name "mydb" \
    --env "POSTGRES_DB=app" \
    --env "POSTGRES_USER=user" \
    --env "POSTGRES_PASSWORD=password"

# Connect application to database
msb exe python \
    --link "mydb:db" \
    --env "DATABASE_URL=postgresql://user:password@db:5432/app" \
    --volume "./app:/workspace" \
    -- python app.py
```

### Load Balancing

```bash
# Start multiple app instances
msb exe python --name "app1" --port "8001:8000" --volume "./app:/workspace" -- python app.py
msb exe python --name "app2" --port "8002:8000" --volume "./app:/workspace" -- python app.py
msb exe python --name "app3" --port "8003:8000" --volume "./app:/workspace" -- python app.py

# Start load balancer
msb exe nginx \
    --port "80:80" \
    --volume "./nginx.conf:/etc/nginx/nginx.conf" \
    --link "app1:app1" \
    --link "app2:app2" \
    --link "app3:app3"
```

## Performance Examples

### Resource Limits

```bash
# Memory-limited container
msb exe python \
    --memory 512 \
    --volume "./app:/workspace" \
    -- python memory_intensive_app.py

# CPU-limited container
msb exe python \
    --cpus 0.5 \
    --volume "./app:/workspace" \
    -- python cpu_intensive_app.py

# Combined limits
msb exe python \
    --memory 1024 \
    --cpus 2.0 \
    --volume "./app:/workspace" \
    -- python balanced_app.py
```

### Monitoring

```bash
# Run with resource monitoring
msb exe python \
    --volume "./app:/workspace" \
    --name "monitored-app" \
    -- python app.py

# Check resource usage (in another terminal)
msb status monitored-app
```

## Security Examples

### Read-Only Mounts

```bash
# Mount source code as read-only
msb exe python \
    --volume "./src:/workspace/src:ro" \
    --volume "./data:/workspace/data" \
    -- python /workspace/src/app.py
```

### Environment Variable Security

```bash
# Use environment file
echo "API_KEY=secret123" > .env
msb exe python \
    --env-file ".env" \
    --volume "./app:/workspace" \
    -- python app.py

# Pass secrets securely
msb exe python \
    --env "DATABASE_PASSWORD=$(cat /path/to/secret)" \
    --volume "./app:/workspace" \
    -- python app.py
```

## Troubleshooting Examples

### Debug Mode

```bash
# Run with debug output
msb exe python \
    --volume "./app:/workspace" \
    --env "DEBUG=true" \
    --log-level debug \
    -- python app.py

# Interactive debugging
msb exe python \
    --volume "./app:/workspace" \
    --interactive \
    -- python -i app.py
```

### Health Checks

```bash
# Run with health check
msb exe python \
    --volume "./app:/workspace" \
    --health-cmd "curl -f http://localhost:8000/health" \
    --health-interval 30s \
    -- python app.py
```

## Best Practices

### Project Structure

```
my-project/
├── Sandboxfile          # Project configuration
├── .env.example         # Environment template
├── docker-compose.yml   # Alternative orchestration
├── src/                 # Source code
├── tests/               # Test files
├── data/                # Data files
├── scripts/             # Utility scripts
├── docs/                # Documentation
└── menv/                # Sandbox state (auto-generated)
```

### Configuration Management

- Use environment variables for configuration
- Keep secrets out of the Sandboxfile
- Use `.env` files for local development
- Document required environment variables

### Resource Management

- Set appropriate memory and CPU limits
- Use volume mounts for persistent data
- Clean up unused resources regularly
- Monitor resource usage

### Development Workflow

- Use projects for complex applications
- Leverage scripts for common tasks
- Use dependencies to manage startup order
- Test in isolated environments

## Next Steps

- [**Projects**](../projects/index.md) - Learn about project-based development
- [**CLI Reference**](../cli/index.md) - Complete command documentation
- [**SDKs**](../sdks/index.md) - Programmatic sandbox management

---

:bulb: **Tip**: Start with simple examples and gradually build more complex configurations as you become familiar with Microsandbox.
