use std::fs;
use std::path::Path;

pub fn resolve_ld_command(cc: &str, ld: &str) -> String {
	let ld_name = Path::new(ld)
		.file_name()
		.and_then(|s| s.to_str())
		.unwrap_or(ld);

	match ld_name {
		"mold" => format!("{} -fuse-ld=mold", cc),
		"ld.lld" | "lld" => format!("{} -fuse-ld=lld", cc),
		"ld.bfd" | "bfd" => format!("{} -fuse-ld=bfd", cc),
		"ld.gold" | "gold" => format!("{} -fuse-ld=gold", cc),
		_ => cc.to_string(),
	}
}

pub fn object_name_for(src: &str) -> String {
	let file_name = Path::new(src)
		.file_name()
		.and_then(|s| s.to_str())
		.unwrap_or(src);

	format!("{}.o", file_name.strip_suffix(".c").unwrap_or(file_name))
}

pub fn ensure_target_dir(name: &str) -> std::io::Result<String> {
	let target_dir = format!("./.basalt/targets/{}", name);
	fs::create_dir_all(&target_dir)?;
	Ok(target_dir)
}

pub fn include_flags(include_directories: &[String], root_var: &str) -> Vec<String> {
	include_directories
		.iter()
		.map(|path| format!("-I{}/{}", root_var, path))
		.collect()
}

pub fn define_flags(defines: &[String]) -> Vec<String> {
	defines.iter().map(|name| format!("-D{}", name)).collect()
}

pub fn makefile_compile_rules(name: &str, sources: &[String]) -> (Vec<String>, Vec<String>, String) {
	let mut source_vars = Vec::new();
	let mut obj_vars = Vec::new();
	let mut compile_rules = String::new();

	for src in sources {
		let clean_src = src.as_str();
		source_vars.push(format!("$(SOURCES_ROOT)/{}", clean_src));

		let obj_name = object_name_for(clean_src);
		let obj_path = format!("$(SOURCES_ROOT)/.basalt/targets/{}/{}", name, obj_name);
		obj_vars.push(obj_path.clone());

		compile_rules.push_str(&format!("{}: $(SOURCES_ROOT)/{}\n", obj_path, clean_src));
		compile_rules.push_str("\t@mkdir -p $(@D)\n");
		compile_rules.push_str(&format!("\t@printf \"  $(C_GREEN)[CC]$(C_RESET)       %s\\n\" \"{}\"\n", clean_src));
		compile_rules.push_str(&format!("\t@$({0}_CC) $({0}_CFLAGS) -c $< -o $@\n\n", name));
	}

	(source_vars, obj_vars, compile_rules)
}

pub fn ninja_compile_build_lines(name: &str, sources: &[String]) -> (Vec<String>, String) {
	let mut obj_paths = Vec::new();
	let mut content = String::new();

	for src in sources {
		let clean_src = src.as_str();
		let obj_name = object_name_for(clean_src);
		let obj_path = format!("$builddir/targets/{}/{}", name, obj_name);
		obj_paths.push(obj_path.clone());

		content.push_str(&format!("build {}: cc $sources_root/{}\n\n", obj_path, clean_src));
	}

	(obj_paths, content)
}
