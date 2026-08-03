use std::fs;
use crate::executable::Executable;
use crate::generators::common::{
	ensure_target_dir, resolve_ld_command, include_flags, makefile_compile_rules,
};

pub fn generate_executable_makefile(exec: &Executable) -> std::io::Result<()> {
	let target_dir = ensure_target_dir(&exec.name)?;

	let mut content = String::new();
	let name = &exec.name;

	let ld_cmd = resolve_ld_command(&exec.ccld, &exec.ld);

	content.push_str("# Toolchain\n");
	content.push_str(&format!("{}_CC := {} -std={}\n", name, exec.cc, exec.c_standard));
	content.push_str(&format!("{}_CXX := {} -std={}\n", name, exec.cxx, exec.cxx_standard));
	content.push_str(&format!("{}_LD := {}\n", name, ld_cmd));
	content.push_str(&format!("{}_AR := {}\n\n", name, exec.ar));

	content.push_str("# Settings\n");

	let dep_cflags: Vec<&str> = exec.dependencies.iter().map(|d| d.cflags.as_str()).collect();
	let dep_ldflags: Vec<&str> = exec.dependencies.iter().map(|d| d.ldflags.as_str()).collect();

	let local_lib_paths: Vec<String> = exec.dependencies.iter()
		.filter_map(|d| d.local_name.as_ref())
		.map(|local_name| format!("-L$(SOURCES_ROOT)/.basalt/targets/{}", local_name))
		.collect();

	let local_lib_outputs: Vec<String> = exec.dependencies.iter()
		.filter_map(|d| d.local_name.as_ref())
		.map(|local_name| format!("$({}_OUTPUT)", local_name))
		.collect();

	let include_directory_flags = include_flags(&exec.include_directories, "$(SOURCE_ROOT)");

	content.push_str(&format!("{}_CFLAGS := -MMD -MP {} {}\n", name, dep_cflags.join(" "), include_directory_flags.join(" ")));
	content.push_str(&format!("{}_CXXFLAGS := -MMD -MP {} {}\n", name, dep_cflags.join(" "), include_directory_flags.join(" ")));
	content.push_str(&format!(
		"{}_LDFLAGS := -L. {} {}\n\n",
		name,
		local_lib_paths.join(" "),
		dep_ldflags.join(" ")
	));

	content.push_str("# Sources\n");
	let (source_vars, obj_vars, compile_rules) = makefile_compile_rules(name, &exec.sources);

	content.push_str(&format!("{}_SOURCES := {}\n", name, source_vars.join(" ")));
	content.push_str(&format!("{}_OBJS := {}\n\n", name, obj_vars.join(" ")));

	content.push_str("# Dependency files\n");
	content.push_str(&format!("{}_DEPS := $({}_OBJS:.o=.d)\n\n", name, name));

	content.push_str("# Output\n");
	content.push_str(&format!("{}_OUTPUT = {}\n\n", name, name));

	content.push_str(&format!("ALL_CLEAN_FILES += $({0}_OUTPUT) $({0}_OBJS) $({0}_DEPS)\n\n", name));

	content.push_str("# Compile rules\n");
	content.push_str(&compile_rules);

	content.push_str("# Link\n");
	content.push_str(&format!(
		"$({0}_OUTPUT): $({0}_OBJS) {1}\n",
		name,
		local_lib_outputs.join(" ")
	));
	content.push_str("\t@mkdir -p $(@D)\n");
	content.push_str(&format!("\t@printf \"  $(C_CYAN)[LD]$(C_RESET)       %s\\n\" \"{0}\"\n", name));
	content.push_str(&format!("\t@$({0}_LD) $({0}_OBJS) -o $({0}_OUTPUT) $({0}_LDFLAGS)\n\n", name));

	content.push_str(&format!("-include $({}_DEPS)\n", name));

	fs::write(format!("{}/Makefile", target_dir), content)?;
	Ok(())
}
