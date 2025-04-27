#[path = "mod.rs"]
mod msb;

use clap::{CommandFactory, Parser};
use microsandbox_cli::{
    AnsiStyles, MicrosandboxArgs, MicrosandboxCliResult, MicrosandboxSubcommand, ServerSubcommand,
};
use microsandbox_core::management::{image, orchestra};
use msb::handlers;

//--------------------------------------------------------------------------------------------------
// Constants
//--------------------------------------------------------------------------------------------------

const SHELL_SCRIPT: &str = "shell";

//--------------------------------------------------------------------------------------------------
// Functions: main
//--------------------------------------------------------------------------------------------------

#[tokio::main]
async fn main() -> MicrosandboxCliResult<()> {
    // Parse command line arguments
    let args = MicrosandboxArgs::parse();

    handlers::log_level(&args);
    tracing_subscriber::fmt::init();

    // Print version if requested
    if args.version {
        println!("{}", format!("v{}", env!("CARGO_PKG_VERSION")).literal());
        return Ok(());
    }

    match args.subcommand {
        Some(MicrosandboxSubcommand::Init {
            path,
            path_with_flag,
        }) => {
            handlers::init_subcommand(path, path_with_flag).await?;
        }
        Some(MicrosandboxSubcommand::Add {
            sandbox,
            build,
            group,
            names,
            image,
            memory,
            cpus,
            volumes,
            ports,
            envs,
            env_file,
            depends_on,
            workdir,
            shell,
            scripts,
            start,
            imports,
            exports,
            scope,
            path,
            config,
        }) => {
            handlers::add_subcommand(
                sandbox, build, group, names, image, memory, cpus, volumes, ports, envs, env_file,
                depends_on, workdir, shell, scripts, start, imports, exports, scope, path, config,
            )
            .await?;
        }
        Some(MicrosandboxSubcommand::Remove {
            sandbox,
            build,
            group,
            names,
            path,
            config,
        }) => {
            handlers::remove_subcommand(sandbox, build, group, names, path, config).await?;
        }
        Some(MicrosandboxSubcommand::List {
            sandbox,
            build,
            group,
            path,
            config,
        }) => {
            handlers::list_subcommand(sandbox, build, group, path, config).await?;
        }
        Some(MicrosandboxSubcommand::Pull {
            image,
            image_group,
            name,
            layer_path,
        }) => {
            image::pull(name, image, image_group, layer_path).await?;
        }
        Some(MicrosandboxSubcommand::Run {
            sandbox,
            build,
            name,
            path,
            config,
            detach,
            exec,
            args,
        }) => {
            handlers::run_subcommand(sandbox, build, name, path, config, detach, exec, args)
                .await?;
        }
        Some(MicrosandboxSubcommand::Shell {
            sandbox,
            build,
            name,
            path,
            config,
            detach,
            args,
        }) => {
            handlers::script_run_subcommand(
                sandbox,
                build,
                name,
                SHELL_SCRIPT.to_string(),
                path,
                config,
                detach,
                args,
            )
            .await?;
        }
        Some(MicrosandboxSubcommand::Exe {
            image: _image,
            name,
            cpus,
            memory,
            volumes,
            ports,
            envs,
            workdir,
            scope,
            exec,
            args,
        }) => {
            handlers::exe_subcommand(
                name, cpus, memory, volumes, ports, envs, workdir, scope, exec, args,
            )
            .await?;
        }
        Some(MicrosandboxSubcommand::Install {
            image: _image,
            name,
            alias,
            cpus,
            memory,
            volumes,
            ports,
            envs,
            workdir,
            scope,
            exec,
            args,
        }) => {
            handlers::install_subcommand(
                name, alias, cpus, memory, volumes, ports, envs, workdir, scope, exec, args,
            )
            .await?;
        }
        Some(MicrosandboxSubcommand::Uninstall { script }) => {
            handlers::uninstall_subcommand(script).await?;
        }
        Some(MicrosandboxSubcommand::Apply { path, config }) => {
            orchestra::apply(path.as_deref(), config.as_deref()).await?;
        }
        Some(MicrosandboxSubcommand::Up {
            sandbox,
            build,
            group,
            names,
            path,
            config,
        }) => {
            handlers::up_subcommand(sandbox, build, group, names, path, config).await?;
        }
        Some(MicrosandboxSubcommand::Down {
            sandbox,
            build,
            group,
            names,
            path,
            config,
        }) => {
            handlers::down_subcommand(sandbox, build, group, names, path, config).await?;
        }
        Some(MicrosandboxSubcommand::Log {
            sandbox,
            build,
            group,
            name,
            path,
            config,
            follow,
            tail,
        }) => {
            handlers::log_subcommand(sandbox, build, group, name, path, config, follow, tail)
                .await?;
        }
        Some(MicrosandboxSubcommand::Clean {
            sandbox,
            name,
            user,
            all,
            path,
            config,
            force,
        }) => {
            handlers::clean_subcommand(sandbox, name, user, all, path, config, force).await?;
        }
        Some(MicrosandboxSubcommand::Self_ { action }) => {
            handlers::self_subcommand(action).await?;
        }
        Some(MicrosandboxSubcommand::Server { subcommand }) => match subcommand {
            ServerSubcommand::Start {
                port,
                namespace_dir,
                dev_mode,
                key,
                detach,
            } => {
                handlers::server_start_subcommand(port, namespace_dir, dev_mode, key, detach)
                    .await?;
            }
            ServerSubcommand::Stop => {
                handlers::server_stop_subcommand().await?;
            }
            ServerSubcommand::Keygen {
                expire,
                namespace,
                all_namespaces,
            } => {
                handlers::server_keygen_subcommand(expire, namespace, all_namespaces).await?;
            }
        },
        Some(_) => (), // TODO: implement other subcommands
        None => {
            MicrosandboxArgs::command().print_help()?;
        }
    }

    Ok(())
}
