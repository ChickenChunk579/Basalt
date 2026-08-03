use std::fs;
use std::path::Path;
use crate::library::Library;

pub fn generate_library_ninja(lib: &Library) -> std::io::Result<()> {
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
    content.push_str(&format!("cc = {}\n", lib.cc));
    content.push_str(&format!("cxx = {}\n", lib.cxx));
    content.push_str(&format!("ld = {}\n\n", ld_cmd));
    content.push_str(&format!("ar = {}\n\n", lib.ar));

    let include_directory_flags: Vec<String> = lib.include_directories.iter()
        .map(|path| format!("-I$sources_root/{}", path))
        .collect();

    let define_flags: Vec<String> = lib.define.iter()
        .map(|name| format!("-D{}", name))
        .collect();

    content.push_str("# Settings\n");
    if lib.is_static {
        content.push_str(&format!("cflags = {} {}\n", include_directory_flags.join(" "), define_flags.join(" ")));
        content.push_str(&format!("cxxflags = {} {}\n", include_directory_flags.join(" "), define_flags.join(" ")));
    } else {
        content.push_str(&format!("cflags = -fPIC {} {}\n", include_directory_flags.join(" "), define_flags.join(" ")));
        content.push_str(&format!("cxxflags = -fPIC {} {}\n", include_directory_flags.join(" "), define_flags.join(" ")));
    }
    content.push_str("ldflags = -L.\n\n");

    content.push_str("# Compile\n");
    let mut obj_paths: Vec<String> = Vec::new();

    for src in &lib.sources {
        let clean_src = src.as_str();
        let file_name = std::path::Path::new(clean_src)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(clean_src);
        let obj_name = file_name.strip_suffix(".c").unwrap_or(file_name).to_string() + ".o";
        let obj_path = format!("$builddir/targets/{}/{}", name, obj_name);
        obj_paths.push(obj_path.clone());

        content.push_str(&format!(
            "build {}: cc $sources_root/{}\n\n",
            obj_path, clean_src
        ));
    }

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
