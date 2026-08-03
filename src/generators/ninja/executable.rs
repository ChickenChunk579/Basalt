use std::fs;
use crate::executable::Executable;
use crate::generators::common::{
	ensure_target_dir, resolve_ld_command, include_flags, define_flags, ninja_compile_build_lines,
};

pub fn generate_executable_ninja(exec: &Executable) -> std::io::Result<()> {
	let target_dir = ensure_target_dir(&exec.name)?;

	let mut content = String::new();
	let name = &exec.name;

	let ld_cmd = resolve_ld_command(&exec.cc, &exec.ld);

	content.push_str("# Toolchain\n");
	content.push_str(&format!("cc = {}\n", exec.cc));
	content.push_str(&format!("cxx = {}\n", exec.cxx));
	content.push_str(&format!("ld = {}\n\n", ld_cmd));

	let dep_cflags: Vec<String> = exec.dependencies.iter()
		.map(|d| d.cflags.replace("$(SOURCES_ROOT)", "$sources_root"))
		.collect();
	let dep_ldflags: Vec<String> = exec.dependencies.iter()
		.map(|d| d.ldflags.replace("$(SOURCES_ROOT)", "$sources_root"))
		.collect();

	let local_lib_paths: Vec<String> = exec.dependencies.iter()
		.filter_map(|d| d.local_name.as_ref())
		.map(|local_name| format!("-L$sources_root/.basalt/targets/{}", local_name))
		.collect();

	let local_lib_outputs: Vec<String> = exec.dependencies.iter()
		.filter_map(|d| d.local_name.as_ref())
		.map(|local_name| format!("$builddir/lib{0}.a", local_name))
		.collect();

	let include_directory_flags = include_flags(&exec.include_directories, "$sources_root");
	let defines = define_flags(&exec.define);

	content.push_str("# Settings\n");
	content.push_str(&format!("cflags = {} {} {}\n", dep_cflags.join(" "), include_directory_flags.join(" "), defines.join(" ")));
	content.push_str(&format!("cxxflags = {} {} {}\n", dep_cflags.join(" "), include_directory_flags.join(" "), defines.join(" ")));
	content.push_str(&format!(
		"ldflags = -L. {} {}\n\n",
		local_lib_paths.join(" "),
		dep_ldflags.join(" ")
	));

	content.push_str("# Compile\n");
	let (obj_paths, compile_lines) = ninja_compile_build_lines(name, &exec.sources);
	content.push_str(&compile_lines);

	content.push_str("# Link\n");
	content.push_str(&format!(
		"build $builddir/{0}: link_exe {1} | {2}\n",
		name,
		obj_paths.join(" "),
		local_lib_outputs.join(" ")
	));

	fs::write(format!("{}/build.ninja", target_dir), content)?;
	Ok(())
}
