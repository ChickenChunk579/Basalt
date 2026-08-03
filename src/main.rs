mod cli;
mod build;
mod dependency;
mod executable;
mod library;
mod js_api;
mod manifest;
mod generators;

use std::fs;
use std::io::Write;
use std::process::Command;
use std::env;
use chrono::Local;
use quick_js::{Context, console::LogConsole};
use log;
use std::path::Path;
use std::collections::HashMap;
use cli::{Cli, Commands};
use clap::Parser;


fn load_option_cache(path: &Path) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Ok(contents) = fs::read_to_string(path) {
        for line in contents.lines() {
            if let Some((k, v)) = line.split_once('=') {
                map.insert(k.to_string(), v.to_string());
            }
        }
    }
    map
}

fn save_option_cache(path: &Path, map: &HashMap<String, String>) {
    let mut out = String::new();
    for (k, v) in map {
        out.push_str(&format!("{}={}\n", k, v));
    }
    let _ = fs::create_dir_all(path.parent().unwrap());
    let _ = fs::write(path, out);
}

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .format(|buf, record| {
            let level_style = buf.default_level_style(record.level());

            writeln!(
                buf,
                "[{} {level_style}{}{level_style:#}] {}",
                Local::now().format("%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .init();

    let mut cli = Cli::parse();

    let cache_path = Path::new(".basalt/options.cache");
    let mut cached = load_option_cache(cache_path);
    
    for opt in &cli.options {
        if let Some((k, v)) = opt.split_once('=') {
            cached.insert(k.to_string(), v.to_string());
        }
    }
    
    save_option_cache(cache_path, &cached);

	cli.options = cached.into_iter().map(|(k, v)| format!("{}={}", k, v)).collect();
    

    if matches!(cli.command, Some(Commands::DistClean)) {
        log::info!("deleting local .basalt...");
        let _ = fs::remove_dir_all(".basalt");

        if let Ok(entries) = fs::read_dir("bedrock-packages") {
            let current_exe = env::current_exe().unwrap();

            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir() && path.join("basalt.config.js").exists() {
                        log::info!("cleaning package {:?}", path);

                        let _ = Command::new(&current_exe)
                            .arg("dist-clean")
                            .env("BASALT_PACKAGE", "1")
                            .current_dir(path)
                            .status();
                    }
                }
            }
        }

        std::process::exit(0);
    }

    if std::env::var("BASALT_PACKAGE").is_ok() {
        if matches!(cli.command, Some(Commands::Run { .. })) {
            cli.command = Some(Commands::Build);
        }
    }

    let context = Context::builder().console(LogConsole).build().expect("Failed to initialize QuickJS");
    js_api::register(&context, Some(&cli));

    log::info!("building dependencies...");
    build::build_deps();

    let config = fs::read_to_string("./basalt.config.js")
        .expect("Unable to read basalt.config.js");

    context.eval(&config).unwrap();
    let result = context.eval("build(globalThis.b);").unwrap();

    let targets = manifest::parse_targets(result);
    
    build::configure(&targets, &cli);

    build::build(&targets, &cli);

    build::run(&targets, &cli);
}
