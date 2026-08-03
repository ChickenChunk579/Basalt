mod dependency;
mod executable;
mod library;
mod js_api;
mod manifest;
mod generators;

use std::fs;
use std::process::Command;
use std::time::Instant;
use std::io::Write;
use std::path::Path;
use chrono::Local;
use quick_js::Context;
use manifest::Target;
use generators::Generator;
use generators::makefile::MakefileGenerator;
use generators::ninja::NinjaGenerator;
use generators::compile_commands::CompileCommandsGenerator;
use sha2::{Sha256, Digest};
use log;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum GeneratorChoice {
    Ninja,
    Make,
}

#[derive(Parser, Debug)]
#[command(name = "basalt", version, about = "A clean, simple and fast build system")]
pub struct Cli {
    #[arg(
        short = 'g', 
        long = "generator", 
        value_enum,
        global = true
    )]
    pub generator: Option<GeneratorChoice>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}


#[derive(Subcommand, Debug, Clone, PartialEq)]
pub enum Commands {
	/// Generate build files and build the project
    Build,
    /// Clean generated artifacts, like object files and output files, from the build directory
    Clean,
    /// Fully clear the .basalt folder
    DistClean,
    /// Generate build files, build the project and run a target by name
	#[command(trailing_var_arg = true, allow_hyphen_values = true)]
    Run {
        target: String,
        args: Vec<String>,
    },
}

fn configure(targets: &Vec<Target>, cli: &Cli) {
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

fn build(_targets: &Vec<Target>, cli: &Cli) {
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


fn run(_targets: &Vec<Target>, cli: &Cli) {
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

	let cli = Cli::parse();
	if let Some(cmd) = cli.command.clone() && cmd == Commands::DistClean {
		log::info!("deleting .basalt...");
		fs::remove_dir_all(".basalt").unwrap();
		std::process::exit(0);
	}
    let context = Context::new().expect("Failed to initialize QuickJS");

    js_api::register(&context);

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
        configure(&targets, &cli);
    } else {
        log::set_max_level(log::LevelFilter::Debug);
    }

    build(&targets, &cli);

    if should_configure {
        if let Some(parent) = sha_stamp.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(sha_stamp, &hash_hex).unwrap();
        log::info!("Configuration state saved successfully.");
    }

	run(&targets, &cli);	
}

