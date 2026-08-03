use std::fs;
use crate::library::Library;
use crate::generators::common::{
	ensure_target_dir, resolve_ld_command, include_flags, define_flags, ninja_compile_build_lines,
};

pub fn generate_library_ninja(lib: &Library) -> std::io::Result<()> {
	let target_dir = ensure_target_dir(&lib.name)?;

	let mut content = String::new();
	let name = &lib.name;

	let ld_cmd = resolve_ld_command(&lib.ccld, &lib.ld);

	content.push_str("# Toolchain\n");
	content.push_str(&format!("cc = {}\n", lib.cc));
	content.push_str(&format!("cxx = {}\n", lib.cxx));
	content.push_str(&format!("ld = {}\n\n", ld_cmd));
	content.push_str(&format!("ar = {}\n\n", lib.ar));

	let include_directory_flags = include_flags(&lib.include_directories, "$sources_root");
	let defines = define_flags(&lib.define);

	content.push_str("# Settings\n");
	if lib.is_static {
		content.push_str(&format!("cflags = {} {}\n", include_directory_flags.join(" "), defines.join(" ")));
		content.push_str(&format!("cxxflags = {} {}\n", include_directory_flags.join(" "), defines.join(" ")));
	} else {
		content.push_str(&format!("cflags = -fPIC {} {}\n", include_directory_flags.join(" "), defines.join(" ")));
		content.push_str(&format!("cxxflags = -fPIC {} {}\n", include_directory_flags.join(" "), defines.join(" ")));
	}
	content.push_str("ldflags = -L.\n\n");

	content.push_str("# Compile\n");
	let (obj_paths, compile_lines) = ninja_compile_build_lines(name, &lib.sources);
	content.push_str(&compile_lines);

	content.push_str("# Archive / Link\n");
	if lib.is_static {
		let output = format!("$builddir/lib{}.a", name);
		content.push_str(&format!("build {}: ar {}\n", output, obj_paths.join(" ")));
	} else {
		let output = format!("$builddir/lib{}.so", name);
		content.push_str(&format!(
			"build {}: link_shared {}\n",
			output, obj_paths.join(" ")
		));
	}

	content.push_str(&format!(
		"\nbuild {0}: phony $builddir/lib{0}{1}\n",
		name,
		if lib.is_static { ".a" } else { ".so" }
	));

	fs::write(format!("{}/build.ninja", target_dir), content)?;
	Ok(())
}
