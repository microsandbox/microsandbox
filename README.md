<a href="./#gh-dark-mode-only" target="_blank">
<img width="100%" src="./assets/microsandbox-banner-xl-dark.png" alt="microsandbox-banner-xl-dark">
</a>
<a href="./#gh-light-mode-only" target="_blank">
<img width="100%" src="./assets/microsandbox-banner-xl.png" alt="microsandbox-banner-xl">
</a>

<div align="center"><b>———&nbsp;&nbsp;&nbsp;easy secure execution of untrusted user/ai code&nbsp;&nbsp;&nbsp;———</b></div>

<br />
<br />

<div align='center'>
  <a href="https://discord.gg/T95Y3XnEAK" target="_blank">
    <img src="https://img.shields.io/badge/discord -%2300acee.svg?color=mediumslateblue&style=for-the-badge&logo=discord&logoColor=white" alt=discord style="margin-bottom: 5px;"/>
  </a>
  <a href="https://docs.microsandbox.dev" target="_blank">
    <img src="https://img.shields.io/badge/documentation-%2300acee.svg?color=ff4500&style=for-the-badge&logo=gitbook&logoColor=white" alt=documentation style="margin-bottom: 5px;"/>
  </a>
  <a href="https://x.com/microsandbox" target="_blank">
    <img src="https://img.shields.io/badge/twitter-%2300acee.svg?color=000000&style=for-the-badge&logo=x&logoColor=white" alt=twitter style="margin-bottom: 5px;"/>
  </a>
</div>

# <sub><img height="18" src="https://octicons-col.vercel.app/question/A770EF">&nbsp;&nbsp;WHY MICROSANDBOX?</sub>

Ever needed to run code you don't fully trust? Whether it's AI-generated code, user submissions, or experimental code, the traditional options all have serious drawbacks:

- **Running locally** - One malicious script and your entire system is compromised
- **Using containers** - Shared kernels mean sophisticated attacks can still break out
- **Traditional VMs** - Waiting 10+ seconds for a VM to boot kills productivity and performance
- **Cloud solutions** - Can get expensive fast and at the whim of the cloud provider

**microsandbox** combines the best of all worlds:

- [x] <span><img height="15" src="https://octicons-col.vercel.app/shield-lock/A770EF">&nbsp;&nbsp;<strong>Bulletproof Security</strong> - True VM isolation with separate kernels</span>
- [x] <span><img height="15" src="https://octicons-col.vercel.app/zap/A770EF">&nbsp;&nbsp;<strong>Instant Startup</strong> - Boot times under 200ms, not 10+ seconds</span>
- [x] <span><img height="15" src="https://octicons-col.vercel.app/home/A770EF">&nbsp;&nbsp;<strong>Your Infrastructure</strong> - Self-hosted with complete control</span>
- [x] <span><img height="15" src="https://octicons-col.vercel.app/package/A770EF">&nbsp;&nbsp;<strong>OCI Compatible</strong> - Works with standard container images</span>
- [x] <span><img height="15" src="https://octicons-col.vercel.app/dependabot/A770EF">&nbsp;&nbsp;<strong>AI-Ready</strong> - Built-in MCP server for seamless AI integration</span>

<div align='center'>• • •</div>

# <sub><img height="18" src="https://octicons-col.vercel.app/zap/A770EF">&nbsp;&nbsp;QUICK START</sub>

Get started with in easy steps:

<div align="center">
  <video src="https://github.com/user-attachments/assets/14e71fe5-1ac8-4663-8370-08002119e909" width="800" controls>
  </video>

<sup><small><a href="https://asciinema.org/a/itQE92vIJiyq1PAPnaGURzDpv" target="_blank">[ASCIINEMA →]</a></small></sup>

</div>

<div align='center'>
  <img src="https://img.shields.io/badge/macos-working-green?style=for-the-badge" alt=macos style="margin-bottom: 5px;"/>
  <img src="https://img.shields.io/badge/linux-working-green?style=for-the-badge" alt=linux style="margin-bottom: 5px;"/>
  <img src="https://img.shields.io/badge/windows-wip-red?style=for-the-badge" alt=windows style="margin-bottom: 5px;"/>
</div>

##

<h3><span>1</span>&nbsp;&nbsp;<img height="13" src="https://octicons-col.vercel.app/north-star/A770EF">&nbsp;&nbsp;Start the Server</h3>

##### Install microsandbox

```sh
curl -sSL https://get.microsandbox.dev | sh
```

##### And start the server

```sh
msb server start --dev
```

> [!TIP]
>
> microsandbox server is also an [MCP server](./MCP.md), so it works directly with Cursor, Agno, and other MCP-enabled AI tools and agents.
>
> For more information on setting up the server, see the [self-hosting guide](./SELF_HOSTING.md).

##

<h3><span>2</span>&nbsp;&nbsp;<img height="14" src="https://octicons-col.vercel.app/move-to-bottom/A770EF">&nbsp;&nbsp;Install the SDK</h3>

##### Python

```sh
pip install microsandbox
```

##### JavaScript

```sh
npm install microsandbox
```

##### Rust

```sh
cargo add microsandbox
```

> [!NOTE]
> There are [SDKs](./sdk) for other languages as well! Join us in expanding support for your favorite language.

<!--
>
> <div align="left">
>   <a href="./sdk/c"><img src="https://img.shields.io/badge/C-A8B9CC?style=flat-square&logo=c&logoColor=white" alt="C"></a>
>   <a href="./sdk/cpp"><img src="https://img.shields.io/badge/C++-00599C?style=flat-square&logo=c%2B%2B&logoColor=white" alt="C++"></a>
>   <a href="./sdk/crystal"><img src="https://img.shields.io/badge/Crystal-000000?style=flat-square&logo=crystal&logoColor=white" alt="Crystal"></a>
>   <a href="./sdk/csharp"><img src="https://img.shields.io/badge/C%23-239120?style=flat-square&logo=c-sharp&logoColor=white" alt="C#"></a>
>   <a href="./sdk/dart"><img src="https://img.shields.io/badge/Dart-0175C2?style=flat-square&logo=dart&logoColor=white" alt="Dart"></a>
>   <a href="./sdk/elixir"><img src="https://img.shields.io/badge/Elixir-4B275F?style=flat-square&logo=elixir&logoColor=white" alt="Elixir"></a>
>   <a href="./sdk/elm"><img src="https://img.shields.io/badge/Elm-1293D8?style=flat-square&logo=elm&logoColor=white" alt="Elm"></a>
>   <a href="./sdk/erlang"><img src="https://img.shields.io/badge/Erlang-A90533?style=flat-square&logo=erlang&logoColor=white" alt="Erlang"></a>
>   <a href="./sdk/fsharp"><img src="https://img.shields.io/badge/F%23-378BBA?style=flat-square&logo=f-sharp&logoColor=white" alt="F#"></a>
>   <a href="./sdk/go"><img src="https://img.shields.io/badge/Go-00ADD8?style=flat-square&logo=go&logoColor=white" alt="Go"></a>
>   <a href="./sdk/haskell"><img src="https://img.shields.io/badge/Haskell-5D4F85?style=flat-square&logo=haskell&logoColor=white" alt="Haskell"></a>
>   <a href="./sdk/java"><img src="https://img.shields.io/badge/Java-ED8B00?style=flat-square&logo=java&logoColor=white" alt="Java"></a>
>   <a href="./sdk/javascript"><img src="https://img.shields.io/badge/JavaScript-F7DF1E?style=flat-square&logo=javascript&logoColor=black" alt="JavaScript"></a>
>   <a href="./sdk/julia"><img src="https://img.shields.io/badge/Julia-9558B2?style=flat-square&logo=julia&logoColor=white" alt="Julia"></a>
>   <a href="./sdk/kotlin"><img src="https://img.shields.io/badge/Kotlin-0095D5?style=flat-square&logo=kotlin&logoColor=white" alt="Kotlin"></a>
>   <a href="./sdk/lua"><img src="https://img.shields.io/badge/Lua-2C2D72?style=flat-square&logo=lua&logoColor=white" alt="Lua"></a>
>   <a href="./sdk/nim"><img src="https://img.shields.io/badge/Nim-FFE953?style=flat-square&logo=nim&logoColor=black" alt="Nim"></a>
>   <a href="./sdk/objc"><img src="https://img.shields.io/badge/Objective--C-438EFF?style=flat-square&logo=apple&logoColor=white" alt="Objective-C"></a>
>   <a href="./sdk/ocaml"><img src="https://img.shields.io/badge/OCaml-EC6813?style=flat-square&logo=ocaml&logoColor=white" alt="OCaml"></a>
>   <a href="./sdk/php"><img src="https://img.shields.io/badge/PHP-777BB4?style=flat-square&logo=php&logoColor=white" alt="PHP"></a>
>   <a href="./sdk/python"><img src="https://img.shields.io/badge/Python-3776AB?style=flat-square&logo=python&logoColor=white" alt="Python"></a>
>   <a href="./sdk/r"><img src="https://img.shields.io/badge/R-276DC3?style=flat-square&logo=r&logoColor=white" alt="R"></a>
>   <a href="./sdk/ruby"><img src="https://img.shields.io/badge/Ruby-CC342D?style=flat-square&logo=ruby&logoColor=white" alt="Ruby"></a>
>   <a href="./sdk/rust"><img src="https://img.shields.io/badge/Rust-000000?style=flat-square&logo=rust&logoColor=white" alt="Rust"></a>
>   <a href="./sdk/scala"><img src="https://img.shields.io/badge/Scala-DC322F?style=flat-square&logo=scala&logoColor=white" alt="Scala"></a>
>   <a href="./sdk/swift"><img src="https://img.shields.io/badge/Swift-FA7343?style=flat-square&logo=swift&logoColor=white" alt="Swift"></a>
>   <a href="./sdk/zig"><img src="https://img.shields.io/badge/Zig-F7A41D?style=flat-square&logo=zig&logoColor=white" alt="Zig"></a>
> </div>
-->

##

<h3><span>3</span>&nbsp;&nbsp;<img height="14" src="https://octicons-col.vercel.app/file-binary/A770EF">&nbsp;&nbsp;Execute the Code</h3>

`microsandbox` offers a growing list of sandbox environment types optimized for different execution requirements. Choose the appropriate sandbox (e.g., PythonSandbox or NodeSandbox) to run your code in a secure tailored environment.

##### Python

```py
import asyncio
from microsandbox import PythonSandbox

async def main():
    async with PythonSandbox.create(name="test") as sb:
        exec = await sb.run("name = 'Python'")
        exec = await sb.run("print(f'Hello {name}!')")

    print(await exec.output()) # prints Hello Python!

asyncio.run(main())
```

##### JavaScript

```js
import { NodeSandbox } from "microsandbox";

async function main() {
  const sb = await NodeSandbox.create({ name: "test" });

  try {
    let exec = await sb.run("var name = 'JavaScript'");
    exec = await sb.run("console.log(`Hello ${name}!`)");

    console.log(await exec.output()); // prints Hello JavaScript!
  } finally {
    await sb.stop();
  }
}

main().catch(console.error);
```

##### Rust

```rs
use microsandbox::{SandboxOptions, PythonSandbox};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sb = PythonSandbox::create(SandboxOptions::builder().name("test").build()).await?;

    let exec = sb.run(r#"name = "Python""#).await?;
    let exec = sb.run(r#"print(f"Hello {name}!")"#).await?;

    println!("{}", exec.output().await?); // prints Hello Python!

    sb.stop().await?;

    Ok(())
}
```

> [!NOTE]
>
> When you run the code for the first time, it will take a while to download the sandbox image unless you already have it downloaded. After that, it will run much faster.
>
> For more information on how to use the SDK, [check out the SDK README](./sdk/README.md).

### Architecture

###

```mermaid
flowchart TB
    %% ── Client side ──────────────────────────
    subgraph ClientProcess["process"]
        A["Your Business Logic"]
        B["microsandbox SDK"]
        A -->|calls| B
    end

    %% ── Server side ─────────────────────────
    subgraph ServerProcess["process"]
        C["microsandbox server"]
    end
    B -->|sends untrusted code to| C

    %% ── Branching hub ───────────────────────
    D([ ])
    C -->|runs code in| D

    %% ── Individual MicroVMs ────────────────
    subgraph VM1["microVM"]
        VM1S["python environment"]
    end

    subgraph VM2["microVM"]
        VM2S["python environment"]
    end

    subgraph VM3["microVM"]
        VM3S["node environment"]
    end

    D --> VM1S
    D --> VM2S
    D --> VM3S

    %% ── Styling ─────────────────────────────
    style A    fill:#D6EAF8,stroke:#2E86C1,stroke-width:2px,color:#000000
    style B    fill:#D6EAF8,stroke:#2E86C1,stroke-width:2px,color:#000000
    style C    fill:#D5F5E3,stroke:#28B463,stroke-width:2px,color:#000000
    style D    fill:#F4F6F6,stroke:#ABB2B9,stroke-width:2px,color:#000000
    style VM1S fill:#FCF3CF,stroke:#F1C40F,stroke-width:2px,color:#000000
    style VM2S fill:#FCF3CF,stroke:#F1C40F,stroke-width:2px,color:#000000
    style VM3S fill:#FCF3CF,stroke:#F1C40F,stroke-width:2px,color:#000000
```

<br />

<!-- TODO: https://github.com/user-attachments/assets/ba466d45-75dd-45ac-917b-0a56c5742e23 -->

<div align='center'>• • •</div>

# <sub><img height="18" src="https://octicons-col.vercel.app/light-bulb/A770EF">&nbsp;&nbsp;USE CASES</sub>

<a href="https://microsandbox.dev#gh-dark-mode-only" target="_blank"><img align="right" width="400" alt="coding-dark" src="https://github.com/user-attachments/assets/37c14bf1-e2f7-4af3-804e-5901de845715"></a>
<a href="https://microsandbox.dev#gh-light-mode-only" target="_blank"><img align="right" width="400" alt="coding-light" src="https://github.com/user-attachments/assets/1bfe7223-869b-4782-9fce-3620c4400bbf"></a>

### Coding & Dev Environments

Let your AI agents build real apps with professional dev tools. When users ask their AI to create a web app, fix a bug, or build a prototype, it can handle everything from Git operations to dependency management to testing in a protected environment.

Your AI can create complete development environments in milliseconds and run programs with full system access. The fast startup means developers get instant feedback and can iterate quickly. This makes it perfect for AI pair programming, coding education platforms, and automated code generation where quick results matter.

<!-- TODO: <div align="center"><a href="https://microsandbox.dev/docs/examples/coding">✨ See coding examples ✨</a></div> -->

<!-- Transparent pixel to create line break after floating image -->

<img width="2000" height="0" src="https://github.com/user-attachments/assets/ee14e6f7-20b8-4391-9091-8e8e25561929"><br>

<a href="https://microsandbox.dev#gh-dark-mode-only" target="_blank"><img align="left" width="400" alt="data-dark" src="https://github.com/user-attachments/assets/3794e426-a223-4064-8939-025c7bbaf5ea"></a>
<a href="https://microsandbox.dev#gh-light-mode-only" target="_blank"><img align="left" width="400" alt="data-light" src="https://github.com/user-attachments/assets/3a330ea5-85b5-4176-8fe7-a43d59733cf1"></a>

### Data Analysis

Transform raw numbers into meaningful insights with AI that works for you. Your AI can process spreadsheets, create charts, and generate reports safely. Whether it's analyzing customer feedback, sales trends, or research data, everything happens in a protected environment that respects data privacy.

Microsandbox lets your AI work with powerful libraries like NumPy, Pandas, and TensorFlow while creating visualizations that bring insights to life. Perfect for financial analysis tools, privacy-focused data processing, medical research, and any situation where you need serious computing power with appropriate safeguards.

<!-- TODO: <div align="center"><a href="https://microsandbox.dev/docs/examples/data-analysis">📊 Explore data examples 📊</a></div> -->

<!-- Transparent pixel to create line break after floating image -->

<img width="2000" height="0" src="https://github.com/user-attachments/assets/ee14e6f7-20b8-4391-9091-8e8e25561929"><br>

<a href="https://microsandbox.dev#gh-dark-mode-only" target="_blank"><img align="right" width="400" alt="web-dark" src="https://github.com/user-attachments/assets/3048a39a-c3cb-4f6e-9bc0-49b404abed03"></a>
<a href="https://microsandbox.dev#gh-light-mode-only" target="_blank"><img align="right" width="400" alt="web-light" src="https://github.com/user-attachments/assets/e6a01e6d-c23f-4c04-bfbf-3e0cb283e0a9"></a>

### Web Browsing Agent

Build AI assistants that can browse the web for your users. Need to compare prices across stores, gather info from multiple news sites, or automate form submissions? Your AI can handle it all while staying in a contained environment.

With microsandbox, your AI can navigate websites, extract data, fill out forms, and handle logins. It can visit any site and deliver only the useful information back to your application. This makes it ideal for price comparison tools, research assistants, content aggregators, automated testing, and web automation workflows that would otherwise require complex setup.

<!-- TODO: <div align="center"><a href="https://microsandbox.dev/docs/examples/web-browsing">🌐 View web examples 🌐</a></div> -->

<!-- Transparent pixel to create line break after floating image -->

<img width="2000" height="0" src="https://github.com/user-attachments/assets/ee14e6f7-20b8-4391-9091-8e8e25561929"><br>

<a href="https://microsandbox.dev#gh-dark-mode-only" target="_blank"><img align="left" width="400" alt="host-dark" src="https://github.com/user-attachments/assets/3c542e78-b5a0-4525-8a2a-376447d786fd"></a>
<a href="https://microsandbox.dev#gh-light-mode-only" target="_blank"><img align="left" width="400" alt="host-light" src="https://github.com/user-attachments/assets/337b3d5f-9c33-4126-ae55-aca33abbf73e"></a>

### Instant App Hosting

Share working apps and demos in seconds without deployment headaches. When your AI creates a useful tool, calculator, visualization, or prototype, users can immediately access it through a simple link.

Zero-setup deployment means your AI-generated code can be immediately useful without complex configuration. Each app runs in its own protected space with appropriate resource limits, and everything cleans up automatically when no longer needed. Perfect for educational platforms hosting student projects, AI assistants creating live demos, and users needing immediate value.

<!-- TODO: <div align="center"><a href="https://microsandbox.dev/docs/examples/app-hosting">🚀 Try hosting examples 🚀</a></div> -->

<!-- Transparent pixel to create line break after floating image -->

<img width="2000" height="0" src="https://github.com/user-attachments/assets/ee14e6f7-20b8-4391-9091-8e8e25561929"><br>

<div align='center'>• • •</div>

# <sub><img height="18" src="https://octicons-col.vercel.app/device-desktop/A770EF">&nbsp;&nbsp;PROJECTS&nbsp;&nbsp;<sup><sup>B E T A</sup></sup></sub>

Beyond the SDK, microsandbox supports project-based development with the familiar package-manager workflow devs are used to. Think of it like npm or cargo, but for sandboxes!

Create a `Sandboxfile`, define your environments, and manage your sandboxes with simple commands.

<br />

<a href="https://asciinema.org/a/7eOFf2Ovigi473FsKgr3Lpve1" target="_blank"><img src="https://github.com/user-attachments/assets/3a9d1de4-2370-4d5a-a40d-9aa7315aa934" /></a>

##

#### Create a Sandbox Project

```sh
msb init
```

This creates a `Sandboxfile` in the current directory, which serves as the configuration manifest for your sandbox environments.

##

#### Add a Sandbox to the Project

```sh
msb add app \
    --image python \
    --cpus 1 \
    --memory 1024 \
    --start 'python -c "print(\"hello\")"'
```

The command above registers a new sandbox named `app` in your Sandboxfile, configured to use the `python` image.

You should now have a `Sandboxfile` containing a sandbox named **`app`**:

```sh
cat Sandboxfile
```

```yaml
# Sandbox configurations
sandboxes:
  app:
    image: python
    memory: 1024
    cpus: 1
    scripts:
      start: python -c "print(\"hello\")"
```

> [!TIP]
>
> Run `msb <subcommand> --help` to see all the options available for a subcommand.
>
> For example, `msb add --help`.

##

#### Running a Sandbox

##### Run a Sandbox Defined in Your Project

```sh
msb run --sandbox app
```

<div align="center">

_**or**_

</div>

```sh
msr app
```

This executes the default _start_ script of your sandbox. For more control, you can directly specify which script to run — `msr app~start`.

When running project sandboxes, all file changes and installations made inside the sandbox are automatically persisted to the `./menv` directory. This means you can stop and restart your sandbox any time without losing your work. Your development environment will be exactly as you left it.

##### Run an Temporary Sandbox

For experimentation or one-off tasks, temporary sandboxes provide a clean environment that leaves no trace:

```sh
msb exe --image python
```

<div align="center">

_**or**_

</div>

```sh
msx python
```

Temporary sandboxes are perfect for isolating programs you get from the internet. Once you exit the sandbox, all changes are completely discarded.

##

#### Installing Sandboxes

The `msb install` command sets up a sandbox as a system-wide executable. It installs a slim launcher program that allows you to start your sandbox from anywhere in your system with a simple command.

```sh
msb install --image alpine
```

<div align="center">

_**or**_

</div>

```sh
msi alpine
```

After installation, you can start your sandbox by simply typing its name in any terminal:

```sh
alpine
```

This makes frequently used sandboxes incredibly convenient to access — no need to navigate to specific directories or remember complex commands. Just type the sandbox name and it launches immediately with all your configured settings.

> [!TIP]
> You can give your sandbox a descriptive, easy-to-remember name during installation:
>
> ```sh
> msi alpine:20250108 slim-linux
> ```
>
> This allows you to create multiple instances of the same sandbox image with different names and configurations. For example:
>
> - `msi python python-data-science` - A Python environment for data analysis
> - `msi python python-web` - A Python environment for web development
>
> Installed sandboxes maintain their state between sessions, so you can pick up exactly where you left off each time you launch them.

<div align='center'>• • •</div>

# <sub><img height="18" src="https://octicons-col.vercel.app/gear/A770EF">&nbsp;&nbsp;DEVELOPMENT</sub>

Interested in contributing to microsandbox? Check out our [Development Guide](./DEVELOPMENT.md) for instructions on setting up your development environment, building the project, running tests, and creating releases.

For contribution guidelines, please refer to [CONTRIBUTING.md](./CONTRIBUTING.md).

<div align='center'>• • •</div>

# <sub><img height="18" src="https://octicons-col.vercel.app/law/A770EF">&nbsp;&nbsp;LICENSE</sub>

This project is licensed under the [Apache License 2.0](./LICENSE).

<div align='center'>• • •</div>

# <sub><img height="18" src="https://octicons-col.vercel.app/star/A770EF">&nbsp;&nbsp;STAR HISTORY</sub>

We appreciate all the support!

<div align='center'>
  <img src="https://api.star-history.com/svg?repos=microsandbox/microsandbox&type=Date">
</div>

<br />

<div align='center'>• • •</div>
