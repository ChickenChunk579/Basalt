use std::fs;
use std::path::Path;
use crate::manifest::Target;
use crate::generators::Generator;

pub struct CompileCommandsGenerator {}

impl CompileCommandsGenerator {
	pub fn new() -> CompileCommandsGenerator {
		CompileCommandsGenerator {}
	}
}

fn json_escape(s: &str) -> String {
	let mut out = String::with_capacity(s.len());
	for c in s.chars() {
		match c {
			'"' => out.push_str("\\\""),
			'\\' => out.push_str("\\\\"),
			'\n' => out.push_str("\\n"),
			'\t' => out.push_str("\\t"),
			_ => out.push(c),
		}
	}
	out
}

fn json_string_array(items: &[String]) -> String {
	let quoted: Vec<String> = items.iter()
		.map(|i| format!("\"{}\"", json_escape(i)))
		.collect();
	format!("[{}]", quoted.join(", "))
}

fn obj_name_for(src: &str) -> String {
	let file_name = Path::new(src)
		.file_name()
		.and_then(|s| s.to_str())
		.unwrap_or(src);
	file_name.strip_suffix(".c").unwrap_or(file_name).to_string() + ".o"
}

fn make_entry(root: &str, target_name: &str, src: &str, args: Vec<String>) -> String {
	let obj_path = format!(".basalt/targets/{}/{}", target_name, obj_name_for(src));
	let mut full_args = args;
	full_args.push(src.to_string());
	full_args.push("-o".to_string());
	full_args.push(obj_path);

	format!(
		"  {{\n    \"directory\": \"{}\",\n    \"file\": \"{}\",\n    \"arguments\": {}\n  }}",
		json_escape(root),
		json_escape(src),
		json_string_array(&full_args)
	)
}

impl Generator for CompileCommandsGenerator {
	fn generate(&self, targets: &[Target]) -> std::io::Result<()> {
		fs::create_dir_all("./.basalt")?;

		let root = std::env::current_dir()?.to_string_lossy().to_string();
		let mut entries: Vec<String> = Vec::new();

		for target in targets {
			match target {
				Target::Executable(exec) => {
					let mut base_args: Vec<String> = vec![exec.cc.clone(), "-c".to_string()];
					for dep in &exec.dependencies {
						for tok in dep.cflags.split_whitespace() {
							base_args.push(tok.replace("$(SOURCES_ROOT)", &root));
						}
					}
					for path in &exec.include_directories {
						base_args.push(format!("-I{}", path));
					}
					for name in &exec.define {
						base_args.push(format!("-D{}", name));
					}

					for src in &exec.sources {
						entries.push(make_entry(&root, &exec.name, src, base_args.clone()));
					}
				}
				Target::Library(lib) => {
					let mut base_args: Vec<String> = vec![lib.cc.clone(), "-c".to_string()];
					if !lib.is_static {
						base_args.push("-fPIC".to_string());
					}
					for path in &lib.include_directories {
						base_args.push(format!("-I{}", path));
					}

					for src in &lib.sources {
						entries.push(make_entry(&root, &lib.name, src, base_args.clone()));
					}
				}
			}
		}

		let json = format!("[\n{}\n]\n", entries.join(",\n"));
		fs::write("./.basalt/compile_commands.json", json)?;
		Ok(())
	}
}
