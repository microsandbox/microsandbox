<div align="centre">Microsandbox vs. Docker</div>

### 1  What’s the Real Difference?

At their core, Microsandbox and Docker represent two fundamentally different approaches to application isolation:

- **Docker** uses container technology, which provides process-level isolation through Linux namespaces and cgroups. While lightweight and efficient, containers share the host's kernel, creating a larger attack surface for potential security vulnerabilities.

- **Microsandbox** leverages microVM technology (using KVM on Linux and Hypervisor.framework on macOS), providing true hardware-level virtualization. Each sandbox runs its own isolated kernel, creating a much stronger security boundary through hardware virtualization extensions.

This architectural difference becomes crucial when considering security, as illustrated below:

```
(A) Regular container                          (B) Microsandbox microVM
┌──────────────────────┐                       ┌──────────────────────┐
│  Your application    │                       │  Your application    │
├──────────────────────┤                       ├──────────────────────┤
│  Host **kernel**     │◀── escapes here       │  **Guest kernel**    │
└──────────────────────┘    - CVE‑2024‑21626   ├──────────────────────┤
                            - CVE‑2024‑23653   │  VMM (KVM / Apple HV)│
                            - CVE‑2023‑27561   ├──────────────────────┤
                                               │  Host kernel         │ ✔ host is *never*
                                               └──────────────────────┘   directly reachable
```

<br />

<div align="center">

|                     | **Docker container**                     | **Microsandbox microVM**                         |
| ------------------- | ---------------------------------------- | ------------------------------------------------ |
| Shares host kernel? | **Yes** – one kernel for every container | **No** – each VM brings a tiny kernel of its own |
| Escape blast‑radius | Whole host                               | Just the single VM                               |
| Startup time        | 50–150 ms typical                        | < 150 ms                                         |

</div>

<br />

<h4 align="center">CVE Spotlight</h4>

<div align="center">

|  Year | CVE                                                                   | 3‑second summary                                                                            | Who was exposed?             |
| ----: | :-------------------------------------------------------------------- | ------------------------------------------------------------------------------------------- | ---------------------------- |
|  2024 | [**CVE‑2024‑21626**](https://nvd.nist.gov/vuln/detail/CVE-2024-21626) | `runc` let attackers overwrite the `runc` binary on the host, hopping out of the container. | Docker / Kubernetes on Linux |
|  2024 | [**CVE‑2024‑23653**](https://nvd.nist.gov/vuln/detail/CVE-2024-23653) | Mount‑namespace mix‑up gave containers read/write access to host paths.                     | Docker, CRI‑O, containerd    |
|  2023 | [**CVE‑2023‑27561**](https://nvd.nist.gov/vuln/detail/CVE-2023-27561) | `io_uring` exploit escalated from unprivileged container to full root on host.              | Any LTS kernel < 6.2         |

</div>

Microsandbox moves the “line of scrimmage” outward: even **full root inside the guest** remains stuck behind hardware virtualisation (Intel VT‑x, AMD‑V, Apple Hypervisor.framework).

<br />

### 2  Cross‑Platform Parity (Linux & macOS today, Windows coming)

Docker Desktop on macOS quietly spins up a hidden Linux VM. Microsandbox makes that VM **first‑class and identical** on every host, so “works on my machine” truly means _same kernel, same cgroup tree, same filesystems_.

<br />

### 3  Zero‑Daemon Developer Experience

- **No long‑running `dockerd`.** The `msb` CLI launches a microVM only when you ask, then quits.
- **Declarative `Sandboxfile`.** Check it into git → CI/CD uses the _exact_ environment you tested locally.

<br />

### 4  Multiple Containers per MicroVM

Microsandbox lets you boot a microVM and run **several containers inside it** (via containerd / Podman) – perfect when your AI agent needs, say, a Postgres side‑car plus a tiny API server.

> **Nested orchestration** (describing those inner containers _in the same Sandboxfile_) is on the roadmap – stay tuned!

<br />

### 5  When Docker Still Makes Sense

|  Use‑case                                  | Notes                                    |
| ------------------------------------------ | ---------------------------------------- |
| Windows‑native workloads _today_           | Until the Microsandbox Windows CLI ships |
| Deeply entrenched Docker‑Compose workflows | Migrate gradually                        |

<div align='center'>• • •</div>

### Try It Out

For installation, quick‑start code and SDK examples, **jump to the [main README](./README.md#quick-start)**.

If you just want to taste the CLI:

```bash
curl -fsSL https://get.microsandbox.dev | sh                  # installs CLI
msb init                                                      # creates a Sandboxfile project
msb add app -i python --start 'python -c "print(\"hello\")"'  # adds a sandbox
msb run app                                                   # boots your first microVM
```

_Happy hacking and stay safe!_
