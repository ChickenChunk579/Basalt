use clap::{Parser, Subcommand, ValueEnum};

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum GeneratorChoice {
    Ninja,
    Make,
}

#[derive(Parser, Debug, Clone)]
#[command(name = "basalt", version, about = "A clean, simple and fast build system")]
pub struct Cli {
    #[arg(
        short = 'g',
        long = "generator",
        value_enum,
        global = true
    )]
    pub generator: Option<GeneratorChoice>,

    /// Build options (-Okey=value)
    #[arg(
        short = 'O',
        value_name = "KEY=VALUE",
        global = true,
        action = clap::ArgAction::Append,
        allow_hyphen_values = true
    )]
    pub options: Vec<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone, PartialEq)]
pub enum Commands {
    /// Generate build files and build the project
    Build,

    /// Clean generated artifacts
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
