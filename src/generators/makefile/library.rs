use std::fs;
use std::path::Path;
use crate::library::Library;

pub fn generate_library_makefile(lib: &Library) -> std::io::Result<()> {
    let target_dir = format!("./.basalt/targets/{}", lib.name);
    fs::create_dir_all(&target_dir)?;

    let mut content = String::new();
    let name = &lib.name;

    let ld_path = Path::new(&lib.ld);
    let ld_name = ld_path
        .file_name()
        .and_then(|os_str| os_str.to_str())
        .unwrap_or(&lib.ld);

    let ld_cmd = match ld_name {
        "mold" => format!("{} -fuse-ld=mold", lib.cc),
        "ld.lld" | "lld" => format!("{} -fuse-ld=lld", lib.cc),
        "ld.bfd" | "bfd" => format!("{} -fuse-ld=bfd", lib.cc),
        "ld.gold" | "gold" => format!("{} -fuse-ld=gold", lib.cc),
        _ => lib.cc.clone(),
    };
    
    content.push_str("# Toolchain\n");
    content.push_str(&format!("{}_CC := {}\n", name, lib.cc));
    content.push_str(&format!("{}_CXX := {}\n", name, lib.cxx));
    content.push_str(&format!("{}_LD := {}\n", name, ld_cmd));
    content.push_str(&format!("{}_AR := {}\n\n", name, lib.ar));

    let include_directory_flags: Vec<String> = lib.include_directories.iter()
    	.map(|path| format!("-I$(SOURCE_ROOT)/{}", path))
    	.collect();

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
    let mut source_vars = Vec::new();
    let mut obj_vars = Vec::new();
    let mut compile_rules = String::new();

    for src in &lib.sources {
        let clean_src = src.as_str();
        source_vars.push(format!("$(SOURCES_ROOT)/{}", clean_src));
        
        let file_name = std::path::Path::new(clean_src)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(clean_src);
        let obj_name = file_name.strip_suffix(".c").unwrap_or(file_name).to_string() + ".o";
        
        let obj_path = format!("$(SOURCES_ROOT)/.basalt/targets/{}/{}", name, obj_name);
        obj_vars.push(obj_path.clone());

        compile_rules.push_str(&format!("{}: $(SOURCES_ROOT)/{}\n", obj_path, clean_src));
        compile_rules.push_str("\t@mkdir -p $(@D)\n");
        compile_rules.push_str(&format!("\t@printf \"  $(C_GREEN)[CC]$(C_RESET)       %s\\n\" \"{}\"\n", clean_src));
        compile_rules.push_str(&format!("\t@$({0}_CC) $({0}_CFLAGS) -c $< -o $@\n\n", name));
    }

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
