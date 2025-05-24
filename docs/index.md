---
order: 1000
icon: home
tags: [introduction]
---

# INTRODUCTION

!!!primary :icon-zap: Secure Code Execution Made Simple
Run untrusted code with **VM-level isolation** and **lightning-fast startup**. Built for AI agents, developers, and anyone who needs to execute code safely without compromising on speed or security.
!!!

<div align='center'>• • •</div>

### :icon-question: Why microsandbox?

Ever needed to run code you don't fully trust? Whether it's AI-generated scripts, user submissions, or experimental code, the traditional options all have serious drawbacks:

- :icon-x: **Running locally** - One malicious script and your entire system is compromised
- :icon-alert: **Using containers** - Shared kernels mean sophisticated attacks can still break out
- :icon-clock: **Traditional VMs** - Waiting 30+ seconds for a VM to boot kills productivity
- :icon-cloud: **Cloud solutions** - Expensive, slow, and you lose control of your infrastructure

**microsandbox changes the game** by combining the best of all worlds:

{.list-icon}

- :icon-shield-lock: **Bulletproof Security** - True VM isolation with separate kernels
- :icon-zap: **Instant Startup** - Boot times under 500ms, not 30+ seconds
- :icon-home: **Your Infrastructure** - Self-hosted with complete control
- :icon-package: **OCI Compatible** - Works with standard container images
- :icon-dependabot: **AI-Ready** - Built-in MCP server for seamless AI integration

<div align='center'>• • •</div>

### :icon-rocket: Get Running in Minutes

+++ :icon-download: Install

```bash
curl -sSL https://get.microsandbox.dev | sh
```

+++ :icon-play: Start

```bash
msb server start --dev
```

+++ :icon-code: Execute

```python
import asyncio
from microsandbox import PythonSandbox

async def main():
    async with PythonSandbox.create(name="demo") as sb:
        exec = await sb.run("print('🚀 Secure execution!')")
        print(await exec.output())

asyncio.run(main())
```

+++

!!!info AI Integration Ready
microsandbox server speaks [MCP](https://modelcontextprotocol.io) natively - connect it to **Cursor**, **Claude**, or any MCP-compatible AI tool in seconds!

[!ref See how to use microsandbox as a MCP server](/mcp)
!!!

<div align='center'>• • •</div>

### :icon-sparkles-fill: What You Can Build

#### :icon-code: AI Code Execution Platforms

Build AI assistants that can safely execute the code they generate. Whether it's a simple Python script or a complex web application, your AI can run, test, and debug code in real-time without compromising your infrastructure. Perfect for coding assistants, educational platforms, and automated development workflows where AI needs to validate its own output.

#### :icon-graph: Secure Data Analysis Services

Create platforms where users can upload datasets and run custom analysis scripts without security concerns. Support any data science stack - Python with pandas, R for statistics, Julia for high-performance computing - while maintaining complete isolation. Ideal for research institutions, business intelligence platforms, and collaborative data science environments.

#### :icon-globe: Interactive Learning Environments

Deploy instant coding environments for education and training. Students can write, compile, and execute code in any programming language directly through their browser while you maintain complete security isolation. Perfect for coding bootcamps, online computer science courses, competitive programming platforms, and technical interview systems.

#### :icon-server: Microservice Testing & Prototyping

Rapidly prototype and test microservices in isolated environments. Spin up complete application stacks, test API integrations, and validate deployment configurations without affecting your main infrastructure. Great for CI/CD pipelines, integration testing, and proof-of-concept development.

<div align='center'>• • •</div>

### :icon-people: Community

#### Get help, share projects, and stay updated

{.list-icon}

- :icon-comment-discussion: **[Discord](https://discord.gg/T95Y3XnEAK)** - Real-time support and discussions
- :icon-mark-github: **[GitHub](https://github.com/microsandbox/microsandbox)** - Source code, issues, and contributions
- :icon-broadcast: **[Reddit](https://www.reddit.com/r/microsandbox)** - Community showcases and tutorials
- :icon-mention: **[Twitter](https://x.com/microsandbox)** - Latest updates and announcements

Whether you're building your first sandbox or scaling to production, our community is here to help you succeed.
