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
use std::path::Path;
use std::process::Command;
use std::env;
use chrono::Local;
use quick_js::Context;
use sha2::{Sha256, Digest};
use log;

use cli::{Cli, Commands};
use clap::Parser;

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

    // 1. Intercept DistClean early, but handle packages before exiting
    if matches!(cli.command, Some(Commands::DistClean)) {
        log::info!("deleting local .basalt...");
        let _ = fs::remove_dir_all(".basalt");

        // Recursively trigger DistClean on all bedrock packages
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

    let context = Context::new().expect("Failed to initialize QuickJS");
    js_api::register(&context);

    log::info!("building dependencies...");
    build::build_deps();

    let config = fs::read_to_string("./basalt.config.js")
        .expect("Unable to read basalt.config.js");

    let hash_hex: String = Sha256::digest(config.as_bytes())
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect();

    let sha_stamp = Path::new(".basalt/sha256");
    let mut should_configure = true;

    if sha_stamp.is_file() {
        let old_sha = fs::read_to_string(sha_stamp)
                .expect("Unable to read sha stamp?");

        if old_sha == hash_hex {
            should_configure = false;
        }
    } else {
        log::info!("first configure");
    }

    if !should_configure {
        log::set_max_level(log::LevelFilter::Warn);
    }

    context.eval(&config).unwrap();
    let result = context.eval("build(globalThis.b);").unwrap();

    let targets = manifest::parse_targets(result);

    if should_configure {
        build::configure(&targets, &cli);
    } else {
        log::set_max_level(log::LevelFilter::Debug);
    }

    build::build(&targets, &cli);

    if should_configure {
        if let Some(parent) = sha_stamp.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(sha_stamp, &hash_hex).unwrap();
        log::info!("Configuration state saved successfully.");
    }

    build::run(&targets, &cli);
}
