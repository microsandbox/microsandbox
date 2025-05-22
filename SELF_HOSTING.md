# <sub><img height="18" src="https://octicons-col.vercel.app/home/A770EF">&nbsp;&nbsp;SELF HOSTING&nbsp;&nbsp;<sup><sup>B E T A</sup></sup></sub>

To get started, you need to host your own sandbox server. Whether that's on a local machine or in the cloud, it's up to you.

Self hosting lets you manage your own data and code making it easier to comply with security policies. Also, having a sandbox server set up locally allows you to test and move through ideas quickly.

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
> - <a href="https://microsandbox.dev#gh-light-mode-only" target="_blank"><img src="https://cdn.simpleicons.org/apple" height="14"/></a><a href="https://microsandbox.dev#gh-dark-mode-only" target="_blank"><img src="https://cdn.simpleicons.org/apple/white" height="14"/></a> **macOS** — Requires Apple Silicon (M1/M2/M3/M4)
> - <a href="https://microsandbox.dev#gh-light-mode-only" target="_blank"><img src="https://cdn.simpleicons.org/linux/black" height="14"/></a><a href="https://microsandbox.dev#gh-dark-mode-only" target="_blank"><img src="https://cdn.simpleicons.org/linux/white" height="14"/></a> **Linux** <a href="https://github.com/microsandbox/microsandbox/issues/224" target="_blank"><sup><small>[WIP →]</small></sup></a> — KVM virtualization must be enabled

##

#### 2. Start Sandbox Server

```sh
msb server start
```

> [!TIP]
>
> Use the `--detach` flag to run the server in the background.
>
> Run `msb server stop` to stop the server if it's running in the background.
>
> See `msb server --help` for more options.

##

#### 3. Pull SDK Images

```sh
msb pull microsandbox/python
```

```sh
msb pull microsandbox/node
```

This pulls and caches the images for the SDKs to use. It is what allows you to run a `PythonSandbox` or `NodeSandbox`.

##

#### 4. Generate API Key

```sh
msb server keygen --expire 3mo
```

After starting the server and generating your key, configure the environment variables to connect your SDK to your self-hosted sandbox server automatically.

There are just two environment variables to set:

- `MSB_API_KEY` — The API key for your sandbox server
- `MSB_API_URL` — The URL of your sandbox server. You don't need to set this if you are running the server locally at the default port.

##

> [!TIP]
>
> For self-hosting on a cloud provider, refer to our [cloud hosting guide](CLOUD_HOSTING.md) for a list of cloud providers that would support running microsandbox.
