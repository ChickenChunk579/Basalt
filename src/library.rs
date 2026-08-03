use crate::dependency::Dependency;

#[derive(Debug)]
pub struct Library {
	pub name: String,
	pub sources: Vec<String>,
	#[allow(dead_code)]
	pub dependencies: Vec<Dependency>,
	pub include_directories: Vec<String>,
	pub define: Vec<String>,
	pub cc: String,
	pub cxx: String,
	pub ld: String,
	pub ar: String,
	pub is_static: bool,
}
