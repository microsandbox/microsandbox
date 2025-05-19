# <sub><img height="18" src="https://octicons-col.vercel.app/home/A770EF">&nbsp;&nbsp;SELF HOSTING&nbsp;&nbsp;<sup><sup>B E T A</sup></sup></sub>

With self hosting, your data and code stay on your servers making security compliance easy. Also, having a local setup allows you to test and move through ideas fast.

Let's help you start your first self-hosted sandbox server. It's easy!

> [!WARNING]
>
> `microsandbox` is beta software and not ready for production use.

##

#### 1. Install CLI

```sh
curl -sSL https://get.microsandbox.dev | sh
```

This will install the `msb` CLI tool, which helps you manage sandboxes locally.

> [!IMPORTANT]
>
> The CLI is currently only available for macOS and Linux. **[Windows support is coming soon!](https://github.com/microsandbox/microsandbox/issues/47)**
>
> **Platform-specific requirements:**
>
> - <a href="https://microsandbox.dev#gh-light-mode-only" target="_blank"><img src="https://cdn.simpleicons.org/linux/black" height="14"/></a><a href="https://microsandbox.dev#gh-dark-mode-only" target="_blank"><img src="https://cdn.simpleicons.org/linux/white" height="14"/></a> **Linux** — KVM virtualization must be enabled
> - <a href="https://microsandbox.dev#gh-light-mode-only" target="_blank"><img src="https://cdn.simpleicons.org/apple" height="14"/></a><a href="https://microsandbox.dev#gh-dark-mode-only" target="_blank"><img src="https://cdn.simpleicons.org/apple/white" height="14"/></a> **macOS** — Requires Apple Silicon (M1/M2/M3/M4)

##

#### 2. Start Sandbox Server

```sh
msb server start
```

> [!TIP]
>
> Use `--detach` flag to run the server in the background.

##

#### 3. Generate API Key

```sh
msb server keygen
```

##

After starting the server and generating your key, [configure the two environment variables](#1get-api-key) to connect your SDK to your self-hosted sandbox server automatically.

> [!TIP]
>
> Run `msb server stop` to stop the server.
>
> See `msb server --help` to see all the available options.
