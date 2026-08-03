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
