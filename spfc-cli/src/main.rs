use anyhow::{Context, Result};
use clap::{Arg, ArgAction, CommandFactory, FromArgMatches, Parser, Subcommand};
use libloading::{Library, Symbol};
use spfc_abi::{
    BackendInfo, CURRENT_ABI_VERSION, CompileFn, CompileOptions, CompileResult, GetBackendInfoFn,
    GetPluginOptionsFn, KeyValuePair, PluginOption, unpack_result,
};
use std::path::PathBuf;

mod manager;
use manager::PluginManager;

#[derive(Parser, Debug)]
#[command(name = "spfc", version, about = "SimplePixelFont Compiler Toolchain")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    // Compile a SimplePixelFont file using a specified target backend
    Compile {
        /// Input SimplePixelFont file path
        #[arg(short, long)]
        input: String,

        /// Output file path
        #[arg(short, long, default_value_t = String::from("output.ttf"))]
        output: String,

        /// Target compilation format (will look for a backend named spfc-target-<target>),
        /// optionally with version (e.g., ttf, ttf@1.0.0, ttf+1.1)
        #[arg(short, long, default_value_t = String::from("ttf"))]
        target: String,
    },
    /// Install a new target backend
    Install {
        /// The target to install (e.g., ttf or ttf@1.0.0)
        target: String,
    },
    /// List all available targets from the registry
    List,
    /// Update the plugin registry by fetching the latest information from the SimplePixelFont/spfc GitHub repository
    UpdateRegistry,
}

fn find_highest_version(dir: &std::path::Path) -> Option<String> {
    let mut versions = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(name) = entry.file_name().into_string() {
                versions.push(name);
            }
        }
    }
    versions.sort();
    versions.pop()
}

fn find_plugin(target: &str, requested_version: Option<&String>) -> Result<PathBuf> {
    let lib_name = get_library_name(target);

    let dev_paths = vec![
        PathBuf::from(format!("./target/debug/{}", lib_name)),
        PathBuf::from(format!("./target/release/{}", lib_name)),
    ];
    for path in dev_paths {
        if path.exists() {
            log::debug!("Found local development plugin at {:?}", path);
            return Ok(path);
        }
    }

    if let Some(proj_dirs) = directories::ProjectDirs::from("org", "SimplePixelFont", "spfc") {
        let target_base_dir = proj_dirs.data_dir().join("plugins").join(target);

        let version_to_use = match requested_version {
            Some(v) => v.clone(),
            None => find_highest_version(&target_base_dir).context(format!(
                "No installed versions found for target '{}'. Run `spfc install {}`",
                target, target
            ))?,
        };

        let prod_path = target_base_dir.join(&version_to_use).join(&lib_name);
        if prod_path.exists() {
            log::info!("Using {} version {}", target, version_to_use);
            return Ok(prod_path);
        }
    }

    anyhow::bail!("Could not find target backend '{}'.", target)
}

/// Helper to parse "ttf@1.0.0" or "ttf+1.0.0" into ("ttf", Some("1.0.0"))
fn parse_target_version(target_str: &str) -> (String, Option<String>) {
    if let Some((name, version)) = target_str.split_once('@') {
        (name.to_string(), Some(version.to_string()))
    } else if let Some((name, version)) = target_str.split_once('+') {
        (name.to_string(), Some(version.to_string()))
    } else {
        (target_str.to_string(), None)
    }
}

fn get_library_name(target: &str) -> String {
    let base = format!("spfc_target_{}", target);
    if cfg!(target_os = "windows") {
        format!("{}.dll", base)
    } else if cfg!(target_os = "macos") {
        format!("lib{}.dylib", base)
    } else {
        format!("lib{}.so", base)
    }
}

/// Peeks at raw command-line arguments to find the target before clap fully parses them.
fn extract_arg(short: &str, long: &str) -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    for i in 0..args.len() {
        if (args[i] == short || args[i] == long) && i + 1 < args.len() {
            return Some(args[i + 1].clone());
        }
    }
    None
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args: Vec<String> = std::env::args().collect();
    let is_compile = args.len() >= 2 && (args[1] == "compile" || args[1] == "c");

    if !is_compile {
        let cli = Cli::parse();
        let manager = PluginManager::new()?;

        match cli.command {
            Commands::Install { target } => {
                let (t_name, t_ver) = parse_target_version(&target);
                manager.install(&t_name, t_ver).await?;
            }
            Commands::List => {
                let output = manager.list_targets().await?;
                println!("{}", output);
            }
            Commands::UpdateRegistry => {
                manager.update_registry().await?;
            }
            _ => unreachable!(),
        }
        return Ok(());
    }

    let raw_target = extract_arg("-t", "--target").unwrap_or_else(|| "ttf".to_string());
    let (target_name, target_version) = parse_target_version(&raw_target);

    let plugin_path = find_plugin(&target_name, target_version.as_ref())?;

    unsafe {
        let lib = Library::new(&plugin_path)?;

        let get_info: Symbol<GetBackendInfoFn> = lib.get(b"get_backend_info")?;
        let backend_info: BackendInfo<String> = unpack_result(get_info())?;
        if backend_info.abi_version != CURRENT_ABI_VERSION {
            anyhow::bail!("Plugin ABI mismatch.");
        }

        let get_plugin_options: Symbol<GetPluginOptionsFn> = lib.get(b"get_plugin_options")?;
        let plugin_options: Vec<PluginOption<String>> = unpack_result(get_plugin_options())?;

        // Build a dynamic clap Command augmented with the plugin's declared options.
        let mut cmd = Cli::command();

        for option in plugin_options.iter() {
            let name: &'static str = Box::leak(option.name.clone().into_boxed_str());
            let description: &'static str = Box::leak(option.description.clone().into_boxed_str());
            let default_value: &'static str =
                Box::leak(option.default_value.clone().into_boxed_str());

            let mut arg = Arg::new(name)
                .long(name)
                .help(description)
                .action(ArgAction::Set);

            if !option.default_value.is_empty() {
                arg = arg.default_value(default_value);
            } else {
                arg = arg.required(false);
            }

            arg = arg.help_heading("Plugin Options");
            cmd = cmd.arg(arg);
        }

        let matches = cmd.get_matches();
        let args = Cli::from_arg_matches(&matches)?;

        if let Commands::Compile { input, output, .. } = args.command {
            let mut extra_args_vec = Vec::new();
            for option in plugin_options.iter() {
                if let Some(val) = matches.get_one::<String>(&*option.name) {
                    extra_args_vec.push(KeyValuePair {
                        key: option.name.clone(),
                        value: val.to_string(),
                    });
                }
            }

            let compile: Symbol<CompileFn> = lib.get(b"compile")?;
            let compile_options = CompileOptions {
                input: input.clone(),
                output: output.clone(),
                extra_arguments: extra_args_vec,
            }
            .try_into()?;
            let compile_result: CompileResult = unpack_result(compile(compile_options))?;
            if compile_result != CompileResult::Success {
                anyhow::bail!(
                    "Backend compilation failed with result: {:?}",
                    compile_result
                );
            }

            lib.close()?;
        }
    }
    Ok(())
}
