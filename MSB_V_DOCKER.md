# Microsandbox vs. Docker: A New Paradigm for Code Isolation

## 1. Enterprise-Grade Security without Performance Tradeoffs

Microsandbox fundamentally reimagines isolation security through hardware-virtualized boundaries:

- **Docker** uses container technology with process-level isolation through Linux namespaces and cgroups. While efficient, all containers share the host kernel—creating a potential attack vector that has led to numerous CVEs allowing container escapes.

- **Microsandbox** leverages microVM technology (KVM on Linux, Hypervisor.framework on macOS), providing true hardware-level virtualization. Each sandbox runs its own isolated kernel, creating an impenetrable security boundary through CPU virtualization extensions.

```
(A) Regular container                          (B) Microsandbox microVM
┌──────────────────────┐                       ┌──────────────────────┐
│  Your application    │                       │  Your application    │
├──────────────────────┤                       ├──────────────────────┤
│  Host **kernel**     │◀── escapes here       │  **Guest kernel**    │
└──────────────────────┘    - CVE‑2024‑23653   ├──────────────────────┤
                            - CVE‑2024‑21626   │  VMM (KVM / Apple HV)│
                            - CVE‑2023‑27561   ├──────────────────────┤
                            - CVE-2024-26584   │  Host kernel         │ ✔ host is *never*
                                               └──────────────────────┘   directly reachable
```

|                     | **Docker Container**                     | **Microsandbox MicroVM**                         |
| ------------------- | ---------------------------------------- | ------------------------------------------------ |
| Shares host kernel? | **Yes** – one kernel for every container | **No** – each VM brings a tiny kernel of its own |
| Escape blast‑radius | Entire host system                       | Just the single VM                               |
| Startup time        | 50–150 ms typical                        | < 150 ms                                         |

Even with **full root access inside a Microsandbox**, attackers remain contained behind hardware virtualization barriers (Intel VT-x, AMD-V, Apple Hypervisor.framework).

<div align='center'>• • •</div>

## 2. True Cross-Platform Consistency

<div align="center">
  <a href="https://www.youtube.com/watch?v=CBbgmRAg0VM" target="_blank">
    <img src="https://img.youtube.com/vi/CBbgmRAg0VM/maxresdefault.jpg" alt="Docker Platform Inconsistencies Explained" width="2000" />
  </a>
  <p><sub><i><a href="https://www.youtube.com/watch?v=CBbgmRAg0VM" target="_blank">▶️ Watch: Real-world examples of cross-platform inconsistencies with Docker</a></i></sub></p>
</div>

**Docker** provides different environments across platforms:

- Linux: Native containers
- macOS/Windows: Hidden Linux VM running in the background
- Production: Often Linux-based with different kernel settings

**Microsandbox** delivers consistent environments across platforms:

- **Identical kernels everywhere** — same microVM technology on all platforms
- **Deterministic builds** with identical underlying systems
- **"Works on my machine"** actually means something when your app behaves identically

<div align='center'>• • •</div>

## 3. Unified Configuration Model: The Sandboxfile

**Docker** requires managing separate Dockerfile and docker-compose.yml files, creating a disjointed configuration experience and synchronization challenges.

**Microsandbox** consolidates everything into a single, declarative `Sandboxfile`:

```yaml
# Multi-stage build definitions (coming soon)
builds:
  api_base:
    image: "python:3"
    memory: 2048
    cpus: 2
    volumes:
      - "./api/requirements.txt:/build/requirements.txt"
    workdir: "/build"
    steps:
      start: |
        pip install -r requirements.txt && \
        pip freeze > requirements.lock && \
        pip install -r requirements.txt -t /build/dist/packages
    exports:
      packages: "/build/dist/packages"
      lockfile: "/build/requirements.lock"

sandboxes:
  api:
    image: "python:3"
    memory: 1024
    cpus: 1
    volumes:
      - "./api:/app/src"
    ports:
      - "8000:8000"
    envs:
      - "DEBUG=false"
      - "API_PORT=8000"
    depends_on:
      - "database"
    workdir: "/app"
    scripts:
      start: "python -m uvicorn src.main:app --host 0.0.0.0 --port 8000"
      test: "pytest src/tests/"
    scope: "public"
    # Import artifacts from the build stage
    imports:
      api_base:
        packages: "/app/packages"
```

**Benefits of the Sandboxfile approach:**

- **Single source of truth** for both image definitions and runtime configuration
- **Multi-stage builds in one file** — combining Dockerfile's multi-stage capabilities with docker-compose orchestration
- **Asset sharing between stages** via imports/exports — easily pass build artifacts to runtime sandboxes
- **Built-in dependency management** with explicit declaration of service relationships
- **Unified scripting** with named, reusable commands
- **Version control friendly** with a single file to track environment changes
- **Clear build-to-deployment pipeline** defined in the same configuration

<div align='center'>• • •</div>

## 4. Developer Workflow Optimization

**Docker**'s development workflow involves managing long-running daemons, complex volume mounts, and separate configurations for local vs. CI environments.

**Microsandbox** streamlines the development experience:

- **Zero-daemon architecture** — `msb` CLI launches microVMs on demand, no background processes
- **Project-based development** — `msb init` creates a sandbox project
- **Throwaway sandboxes** — `msb exe python` for quick experiments
- **Quick access shortcuts** — `msb install alpine dev-env` that creates system-level commands
- **Simplified network controls** — `local`, `public`, `any`, or `none`

<div align='center'>• • •</div>

## 5. Built for the AI Agent Era

**Docker** was designed for microservices and DevOps workflows.

**Microsandbox** is purpose-built for the emerging AI agent ecosystem:

- **Secure execution boundaries** — hardware-level isolation for untrusted AI-generated code
- **Native MCP integration** — seamless communication with modern AI models
- **Multi-language SDK ecosystem** — consistent APIs across 25+ programming languages

<div align='center'>• • •</div>

## 6. Intuitive Command-Line Interface

**Docker**'s CLI evolved organically with some inconsistencies across commands.

**Microsandbox**'s CLI is streamlined for the modern developer:

```bash
msb init                                       # Create new Sandboxfile project
msb add app -i python:3.11                     # Add a sandbox
msb run app                                    # Run the default script
msb exe python                                 # Quick temporary sandbox
msi alpine my-tool                             # Install as a system command
```

<div align='center'>• • •</div>

## Try Microsandbox Today

For full installation, quick-start code and SDK examples, **jump to the [main README](./README.md#quick-start)**.

_Secure, streamlined, and built for modern development workflows._
