use crate::dependency::Dependency;

#[derive(Debug)]
pub struct Library {
	pub name: String,
	pub sources: Vec<String>,
	#[allow(dead_code)]
	pub dependencies: Vec<Dependency>,
	pub include_directories: Vec<String>,
	pub define: Vec<String>,
	pub cxx_standard: String,
	pub c_standard: String,
	pub cc: String,
	pub ccld: String,
	pub cxx: String,
	pub ld: String,
	pub ar: String,
	pub is_static: bool,
}
