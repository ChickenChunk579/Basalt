use crate::dependency::Dependency;

#[derive(Debug)]
pub struct Executable {
	pub name: String,
	pub sources: Vec<String>,
	pub include_directories: Vec<String>,
	pub dependencies: Vec<Dependency>,
	pub define: Vec<String>,
	pub cc: String,
	pub cxx: String,
	pub ld: String,
	pub ar: String,
}
