# <sub><img height="18" src="https://octicons-col.vercel.app/home/A770EF">&nbsp;&nbsp;SELF HOSTING&nbsp;&nbsp;<sup><sup>B E T A</sup></sup></sub>

To get started, you need to host your own sandbox server. Whether that's on a local machine or in the cloud, it's up to you.

Self hosting lets you manage your own data and code making it easier to comply with security policies. Also, having a sandbox server set up locally allows you to test and move through ideas quickly.

Let's help you start your first self-hosted sandbox server. It's easy!

> **Platform-specific requirements:**
>
> - <a href="https://microsandbox.dev#gh-light-mode-only" target="_blank"><img src="https://cdn.simpleicons.org/apple" height="14"/></a><a href="https://microsandbox.dev#gh-dark-mode-only" target="_blank"><img src="https://cdn.simpleicons.org/apple/white" height="14"/></a> **macOS** — Requires Apple Silicon (M1/M2/M3/M4)
> - <a href="https://microsandbox.dev#gh-light-mode-only" target="_blank"><img src="https://cdn.simpleicons.org/linux/black" height="14"/></a><a href="https://microsandbox.dev#gh-dark-mode-only" target="_blank"><img src="https://cdn.simpleicons.org/linux/white" height="14"/></a>  **Linux** <a href="https://github.com/microsandbox/microsandbox/issues/224" target="_blank"><sup><sup>#224</sup></sup></a> — KVM virtualization must be enabled
> - <a href="https://microsandbox.dev#gh-light-mode-only" target="_blank"><img src="https://github.com/user-attachments/assets/1677b695-e359-4b51-9931-f8f5f9488e71" height="14"/></a><a href="https://microsandbox.dev#gh-dark-mode-only" target="_blank"><img src="https://github.com/user-attachments/assets/e3e5b341-b097-45d9-bea1-eb70e0769340" height="14"/></a> **Windows** <a href="https://github.com/microsandbox/microsandbox/issues/224" target="_blank"> — [Coming soon!](https://github.com/microsandbox/microsandbox/issues/47)

##

#### 1. Install CLI

```sh
curl -sSL https://get.microsandbox.dev | sh
```

This will install the `msb` CLI tool, which helps you manage sandboxes locally.

##

#### 2. Start Sandbox Server

```sh
msb server start
```

> [!TIP]
>
> Use the `--detach` flag to run the server in the background.
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

After generating your key, set the `MSB_API_KEY` environment variable to the generated key.

##

> [!TIP]
>
> For self-hosting on a cloud provider, refer to our [cloud hosting guide](CLOUD_HOSTING.md) for a list of cloud providers that would support running microsandbox.
