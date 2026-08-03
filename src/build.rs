use std::fs;
use std::process::Command;
use std::time::Instant;
use std::env;

use crate::cli::{Cli, Commands, GeneratorChoice};
use crate::manifest::Target;
use crate::generators::Generator;
use crate::generators::makefile::MakefileGenerator;
use crate::generators::ninja::NinjaGenerator;
use crate::generators::compile_commands::CompileCommandsGenerator;

pub fn configure(targets: &Vec<Target>, cli: &Cli) {
    log::info!("configuring...");

    log::info!("targets:");
    for target in targets {
        match target {
            Target::Executable(exec) => log::info!("  {} (exe)", exec.name),
            Target::Library(lib) => log::info!("  {} (lib)", lib.name),
        }
    }

    let generator: Option<Box<dyn Generator>>;

    match cli.generator {
        Some(GeneratorChoice::Ninja) => {
            log::info!("generating for ninja");
            generator = Some(Box::new(NinjaGenerator::new()));
        },
        Some(GeneratorChoice::Make) => {
            log::info!("generating for make");
            generator = Some(Box::new(MakefileGenerator::new()));
        },
        None => {
            log::info!("generating for ninja");
            generator = Some(Box::new(NinjaGenerator::new()));
        }
    }

    generator.unwrap().generate(&targets).unwrap();

    log::info!("creating compile_commands.json...");

    let _ = CompileCommandsGenerator::new().generate(&targets);
}

pub fn build(_targets: &Vec<Target>, cli: &Cli) {
    log::info!("building");

    let threads = num_cpus::get();

    let (tool, dir_flag) = match cli.generator.unwrap_or(GeneratorChoice::Ninja) {
        GeneratorChoice::Ninja => ("ninja", vec!["-C", ".basalt"]),
        GeneratorChoice::Make => ("make", vec!["-C", ".basalt"]),
    };

    let tool_args: Vec<String> = match &cli.command {
        Some(Commands::Build) | None => {
            vec!["all".to_string(), format!("-j{}", threads)]
        },
        Some(Commands::Clean) => match cli.generator.unwrap_or(GeneratorChoice::Ninja) {
            GeneratorChoice::Ninja => vec!["-t".to_string(), "clean".to_string()],
            GeneratorChoice::Make => vec!["clean".to_string()],
        },
        Some(Commands::Run { target: _, args: _ }) => {
            vec!["all".to_string(), format!("-j{}", threads)]
        },
        Some(Commands::DistClean) => unreachable!()
    };

    let start_time = Instant::now();
    let build_status = Command::new(tool)
        .args(&dir_flag)
        .args(&tool_args)
        .status()
        .expect("Failed to execute build tool command");

    if !build_status.success() {
        std::process::exit(build_status.code().unwrap_or(1));
    }
    let duration = start_time.elapsed();
    log::info!("build took {:.2?}", duration);
}

pub fn run(_targets: &Vec<Target>, cli: &Cli) {
    if let Some(Commands::Run { target, args }) = &cli.command {
        log::info!("running target: {}", target);

        let target_binary_path = format!("./.basalt/{}", target);

        let run_status = Command::new(&target_binary_path)
            .args(args)
            .status()
            .expect("Failed to execute target binary");

        if !run_status.success() {
            std::process::exit(run_status.code().unwrap_or(1));
        }
    }
}

pub fn build_deps() {
    let Ok(entries) = fs::read_dir("bedrock-packages") else {
        return;
    };

    let mut targets_to_build = Vec::new();

    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir() {
            let config_path = path.join("basalt.config.js");

            if !config_path.exists() {
                panic!("Missing basalt.config.js in {:?}", path);
            }

            targets_to_build.push(path);
        }
    }

    let current_exe = env::current_exe().unwrap();
    let args: Vec<String> = env::args().skip(1).collect();

    for dir in targets_to_build {
        log::info!("configuring {:?}", dir);

        let status = Command::new(&current_exe)
            .args(&args)
            .current_dir(dir)
            .env("BASALT_PACKAGE", "1")
            .status()
            .unwrap();

        if !status.success() {
            std::process::exit(status.code().unwrap_or(1));
        }
    }
}
