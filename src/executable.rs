use crate::dependency::Dependency;

#[derive(Debug)]
pub struct Executable {
	pub name: String,
	pub sources: Vec<String>,
	pub include_directories: Vec<String>,
	pub dependencies: Vec<Dependency>,
	pub define: Vec<String>,
	pub cxx_standard: String,
	pub c_standard: String,
	pub cc: String,
	pub ccld: String,
	pub cxx: String,
	pub ld: String,
	pub ar: String,
}
