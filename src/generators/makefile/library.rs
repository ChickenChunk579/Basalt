use std::fs;
use crate::library::Library;
use crate::generators::common::{
	ensure_target_dir, resolve_ld_command, include_flags, makefile_compile_rules,
};

pub fn generate_library_makefile(lib: &Library) -> std::io::Result<()> {
	let target_dir = ensure_target_dir(&lib.name)?;

	let mut content = String::new();
	let name = &lib.name;

	let ld_cmd = resolve_ld_command(&lib.ccld, &lib.ld);

	content.push_str("# Toolchain\n");
	content.push_str(&format!("{}_CC := {} -std={}\n", name, lib.cc, lib.c_standard));
	content.push_str(&format!("{}_CXX := {} -std={}\n", name, lib.cxx, lib.cxx_standard));
	content.push_str(&format!("{}_LD := {}\n", name, ld_cmd));
	content.push_str(&format!("{}_AR := {}\n\n", name, lib.ar));

	let include_directory_flags = include_flags(&lib.include_directories, "$(SOURCE_ROOT)");

	content.push_str("# Settings\n");
	if lib.is_static {
		content.push_str(&format!("{}_CFLAGS := -MMD -MP {}\n", name, include_directory_flags.join(" ")));
		content.push_str(&format!("{}_CXXFLAGS := -MMD -MP {}\n", name, include_directory_flags.join(" ")));
	} else {
		content.push_str(&format!("{}_CFLAGS := -MMD -MP -fPIC {}\n", name, include_directory_flags.join(" ")));
		content.push_str(&format!("{}_CXXFLAGS := -MMD -MP -fPIC {}\n", name, include_directory_flags.join(" ")));
	}
	content.push_str(&format!("{}_LDFLAGS := -L.\n\n", name));

	content.push_str("# Sources\n");
	let (source_vars, obj_vars, compile_rules) = makefile_compile_rules(name, &lib.sources);

	content.push_str(&format!("{}_SOURCES := {}\n", name, source_vars.join(" ")));
	content.push_str(&format!("{}_OBJS := {}\n\n", name, obj_vars.join(" ")));

	content.push_str("# Dependency files\n");
	content.push_str(&format!("{}_DEPS := $({}_OBJS:.o=.d)\n\n", name, name));

	content.push_str("# Output\n");
	if lib.is_static {
		content.push_str(&format!("{}_OUTPUT = lib{}.a\n\n", name, name));
	} else {
		content.push_str(&format!("{}_OUTPUT = lib{}.so\n\n", name, name));
	}

	content.push_str(&format!("ALL_CLEAN_FILES += $({0}_OUTPUT) $({0}_OBJS) $({0}_DEPS)\n\n", name));

	content.push_str("# Compile rules\n");
	content.push_str(&compile_rules);

	content.push_str("# Archive / Link\n");
	content.push_str(&format!("{0}: $({0}_OUTPUT)\n\n", name));
	content.push_str(&format!("$({0}_OUTPUT): $({0}_OBJS)\n", name));
	content.push_str("\t@mkdir -p $(@D)\n");

	if lib.is_static {
		content.push_str(&format!("\t@printf \"  $(C_YELLOW)[AR]$(C_RESET)       lib%s.a\\n\" \"{0}\"\n", name));
		content.push_str(&format!("\t@$({0}_AR) rcs $({0}_OUTPUT) $({0}_OBJS)\n\n", name));
	} else {
		content.push_str(&format!("\t@$({0}_LD) -shared $({0}_OBJS) -o $({0}_OUTPUT) $({0}_LDFLAGS)\n\n", name));
		content.push_str(&format!("\t@$({0}_LD) -shared $({0}_LDFLAGS) $({0}_OBJS) -o $({0}_OUTPUT)\n\n", name));
	}

	content.push_str(&format!("-include $({}_DEPS)\n", name));

	fs::write(format!("{}/Makefile", target_dir), content)?;
	Ok(())
}
