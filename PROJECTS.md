# <sub><img height="18" src="https://octicons-col.vercel.app/device-desktop/A770EF">&nbsp;&nbsp;PROJECTS&nbsp;&nbsp;<sup><sup>B E T A</sup></sup></sub>

The `msb` CLI brings the familiar feel of package managers to sandbox development. Think of it like npm or cargo, but for sandboxes! Create a Sandboxfile, define your environments, and manage your sandboxes with simple commands.

> [!WARNING]
>
> `microsandbox` is beta software and not ready for production use.

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
msb r app
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
msb x python
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
msb i alpine
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

<br />

<a href="https://asciinema.org/a/7eOFf2Ovigi473FsKgr3Lpve1" target="_blank"><img src="https://github.com/user-attachments/assets/3a9d1de4-2370-4d5a-a40d-9aa7315aa934" /></a>
