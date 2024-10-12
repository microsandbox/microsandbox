<div align="center">
  <!-- <a href="https://github.com/appcypher/monocore" target="_blank">
    <img src="https://raw.githubusercontent.com/appcypher/monocore/main/assets/a_logo.png" alt="monocore Logo" width="100"></img>
  </a> -->

  <h1 align="center">monocore</h1>
<!--
  <p>
    <a href="https://crates.io/crates/monocore">
      <img src="https://img.shields.io/crates/v/monocore?label=crates" alt="Crate">
    </a>
    <a href="https://github.com/appcypher/monocore/actions?query=">
      <img src="https://github.com/appcypher/monocore/actions/workflows/tests_and_checks.yml/badge.svg" alt="Build Status">
    </a>
    <a href="https://github.com/appcypher/monocore/blob/main/LICENSE">
      <img src="https://img.shields.io/badge/License-Apache%202.0-blue.svg" alt="License">
    </a>
    <a href="https://docs.rs/monocore">
      <img src="https://img.shields.io/static/v1?label=Docs&message=docs.rs&color=blue" alt="Docs">
    </a>
  </p> -->
</div>

**`monocore`** is an open-source self-hostable platform for orchestrating lightweight virtual machines.

It is specifically designed to quickly provision sandboxed environments for executing code and storing other artifacts generated by **AI agents** _and people_.

> [!WARNING]
> This project is in early development and is not yet ready for production use.

##

## Outline

- [Development](#development)
- [License](#license)

## Development

If you want to contribute to monocore, you will need to build the project from source.

### 1. Getting the code

```sh
git clone https://github.com/appcypher/monocore
cd monocore
```

### 2. Building libkrun

monocore uses a modified version of [libkrun][libkrun-repo] as the backend for its microVMs, so you will need to build it first.

To do so, run the following command in the `monocore` directory:

```sh
./build_libkrun.sh
```

> [!NOTE]
> On macOS, you will need to install `krunvm` using Homebrew before you can build libkrun.
>
> ```sh
> brew tap slp/tap
> brew install krunvm
> ```

### 3. Building monocore

```sh
cargo build --release
```

## License

This project is licensed under the [Apache License 2.0](./LICENSE).

[libkrun-repo]: https://github.com/containers/libkrun
