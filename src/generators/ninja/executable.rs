use std::fs;
use std::path::Path;
use crate::executable::Executable;

pub fn generate_executable_ninja(exec: &Executable) -> std::io::Result<()> {
    let target_dir = format!("./.basalt/targets/{}", exec.name);
    fs::create_dir_all(&target_dir)?;

    let mut content = String::new();
    let name = &exec.name;

    let ld_path = Path::new(&exec.ld);
    let ld_name = ld_path
        .file_name()
        .and_then(|os_str| os_str.to_str())
        .unwrap_or(&exec.ld);

    let ld_cmd = match ld_name {
        "mold" => format!("{} -fuse-ld=mold", exec.cc),
        "ld.lld" | "lld" => format!("{} -fuse-ld=lld", exec.cc),
        "ld.bfd" | "bfd" => format!("{} -fuse-ld=bfd", exec.cc),
        "ld.gold" | "gold" => format!("{} -fuse-ld=gold", exec.cc),
        _ => exec.cc.clone(),
    };

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

    let include_directory_flags: Vec<String> = exec.include_directories.iter()
        .map(|path| format!("-I$sources_root/{}", path))
        .collect();

    let define_flags: Vec<String> = exec.define.iter()
        .map(|name| format!("-D{}", name))
        .collect();

    content.push_str("# Settings\n");
    content.push_str(&format!("cflags = {} {} {}\n", dep_cflags.join(" "), include_directory_flags.join(" "), define_flags.join(" ")));
    content.push_str(&format!("cxxflags = {} {} {}\n", dep_cflags.join(" "), include_directory_flags.join(" "), define_flags.join(" ")));
    content.push_str(&format!(
        "ldflags = -L. {} {}\n\n",
        local_lib_paths.join(" "),
        dep_ldflags.join(" ")
    ));



    content.push_str("# Compile\n");
    let mut obj_paths: Vec<String> = Vec::new();

    for src in &exec.sources {
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
