pub mod makefile;
pub mod ninja;
pub mod compile_commands;

use crate::Target;

pub trait Generator {
	fn generate(&self, targets: &[Target]) -> std::io::Result<()>;
}
