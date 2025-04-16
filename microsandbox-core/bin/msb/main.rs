#[path = "mod.rs"]
mod msb;

use clap::{CommandFactory, Parser};
use microsandbox_core::{
    cli::{MicrosandboxArgs, MicrosandboxSubcommand, ServerSubcommand},
    management::{image, orchestra, server},
    MicrosandboxResult,
};
use msb::handlers;

//--------------------------------------------------------------------------------------------------
// Constants
//--------------------------------------------------------------------------------------------------

const START_SCRIPT: &str = "start";
const SHELL_SCRIPT: &str = "shell";

//--------------------------------------------------------------------------------------------------
// Functions: main
//--------------------------------------------------------------------------------------------------

#[tokio::main]
async fn main() -> MicrosandboxResult<()> {
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let args = MicrosandboxArgs::parse();
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
            imports,
            exports,
            reach,
            path,
            config,
        }) => {
            handlers::add_subcommand(
                sandbox, build, group, names, image, memory, cpus, volumes, ports, envs, env_file,
                depends_on, workdir, shell, scripts, imports, exports, reach, path, config,
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
            args,
            path,
            config,
            detach,
            exec,
        }) => {
            handlers::run_subcommand(sandbox, build, name, args, path, config, detach, exec)
                .await?;
        }
        Some(MicrosandboxSubcommand::Start {
            sandbox,
            build,
            name,
            args,
            path,
            config,
            detach,
        }) => {
            handlers::script_run_subcommand(
                sandbox,
                build,
                name,
                START_SCRIPT.to_string(),
                args,
                path,
                config,
                detach,
                None,
            )
            .await?;
        }
        Some(MicrosandboxSubcommand::Shell {
            sandbox,
            build,
            name,
            args,
            path,
            config,
            detach,
        }) => {
            handlers::script_run_subcommand(
                sandbox,
                build,
                name,
                SHELL_SCRIPT.to_string(),
                args,
                path,
                config,
                detach,
                None,
            )
            .await?;
        }
        Some(MicrosandboxSubcommand::Tmp {
            image: _image,
            name,
            cpus,
            memory,
            volumes,
            ports,
            envs,
            workdir,
            exec,
            args,
        }) => {
            handlers::tmp_subcommand(
                name, cpus, memory, volumes, ports, envs, workdir, exec, args,
            )
            .await?;
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
        Some(MicrosandboxSubcommand::Clean { global, all, path }) => {
            handlers::clean_subcommand(global, all, path).await?;
        }
        Some(MicrosandboxSubcommand::Server { subcommand }) => match subcommand {
            ServerSubcommand::Start {
                port,
                path,
                disable_default,
                secure,
                key,
                detach,
            } => {
                handlers::server_start_subcommand(port, path, disable_default, secure, key, detach)
                    .await?;
            }
            ServerSubcommand::Stop => {
                server::stop().await?;
            }
            ServerSubcommand::Keygen { expire } => {
                handlers::server_keygen_subcommand(expire).await?;
            }
        },
        Some(_) => (), // TODO: implement other subcommands
        None => {
            MicrosandboxArgs::command().print_help()?;
        }
    }

    Ok(())
}
