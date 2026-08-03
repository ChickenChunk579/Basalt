use std::collections::HashMap;
use quick_js::{Context, JsValue};
use log;
use crate::Cli;


use crate::dependency::{Dependency, resolve_dependency_pkgconf, resolve_bedrock_package};

pub enum OptionValue {
    Integer(i32),
    Float(f64),
    Boolean(bool),
    String(String),
    None,
}

fn parse_value(raw: &str) -> OptionValue {
    if raw == "null" || raw == "undefined" {
        return OptionValue::None;
    }

    if let Ok(boolean) = raw.parse::<bool>() {
        return OptionValue::Boolean(boolean);
    }

    if let Ok(number) = raw.parse::<i32>() {
        return OptionValue::Integer(number);
    }

    if let Ok(float) = raw.parse::<f64>() {
        return OptionValue::Float(float);
    }

    OptionValue::String(raw.to_string())
}

fn get_option(cli: Option<&Cli>, name: String, default: String) -> OptionValue {
    if let Some(cli) = cli {
        for option in &cli.options {
            if let Some((key, value)) = option.split_once('=') {
                if key == name {
                    return parse_value(value);
                }
            }
        }
    }

    parse_value(&default)
}


pub fn register(context: &Context, cli: Option<&Cli>) {
	let cli_options = cli.cloned();
	
    context.add_callback("__api_option", move |name: String, default: String| {
        match get_option(cli_options.as_ref(), name, default) {
            OptionValue::Integer(value) => JsValue::Int(value),
            OptionValue::Float(value) => JsValue::Float(value),
            OptionValue::Boolean(value) => JsValue::Bool(value),
            OptionValue::String(value) => JsValue::String(value),
            OptionValue::None => JsValue::Null,
        }
    }).unwrap();
	
	context.add_callback("__api_toolchain", |toolchain: HashMap<String, JsValue>| {
		JsValue::Object(toolchain)
	}).unwrap();

	context.add_callback("__api_system_lib", |name: String| {
		let mut dep_obj = HashMap::new();
		dep_obj.insert("cflags".to_string(), JsValue::String("".to_string()));
		dep_obj.insert("ldflags".to_string(), JsValue::String(format!("-l{}", name)));

		log::info!("linking {} (system lib)", name);
		log::info!("  cflags: ");
		log::info!("  ldflags: -l{}", name);

		JsValue::Object(dep_obj)	
	}).unwrap();

	context.add_callback("__api_dependency", |name: String| {
	    if let Some((pkg_name, pkg_lib)) = name.split_once("::") {
	        if let Some(dep) =
	            resolve_bedrock_package(pkg_name.to_string(), pkg_lib.to_string())
	        {
	            log::info!("found {}::{} (bedrock packages)", pkg_name, pkg_lib);
	            log::info!("  cflags: {}", dep.cflags);
	            log::info!("  ldflags: {}", dep.ldflags);
	
	            let mut dep_obj = HashMap::new();
	            dep_obj.insert("cflags".to_string(), JsValue::String(dep.cflags));
	            dep_obj.insert("ldflags".to_string(), JsValue::String(dep.ldflags));
	
	            return JsValue::Object(dep_obj);
	        } else {
	            panic!("no package called {}", pkg_name);
	        }
	    }
	    
	    if let Some(pkgconf_dependency) = resolve_dependency_pkgconf(name.clone()) {
	        log::info!("found {} via pkgconf", name);
	        log::info!("  cflags: {}", pkgconf_dependency.cflags);
	        log::info!("  ldflags: {}", pkgconf_dependency.ldflags);
	
	        let mut dep_obj = HashMap::new();
	        dep_obj.insert(
	            "cflags".to_string(),
	            JsValue::String(pkgconf_dependency.cflags),
	        );
	        dep_obj.insert(
	            "ldflags".to_string(),
	            JsValue::String(pkgconf_dependency.ldflags),
	        );
	
	        JsValue::Object(dep_obj)
	    } else {
	        log::warn!("unable to find dependency {}", name);
	        log::warn!("directly linking.");
	        log::warn!("if this is intentional, use b.systemLib(name)");
	
	        let mut dep_obj = HashMap::new();
	        dep_obj.insert("cflags".to_string(), JsValue::String(String::new()));
	        dep_obj.insert(
	            "ldflags".to_string(),
	            JsValue::String(format!("-l{}", name)),
	        );
	
	        log::info!("found {} (direct link)", name);
	        log::info!("  cflags: ");
	        log::info!("  ldflags: -l{}", name);
	
	        JsValue::Object(dep_obj)
	    }
	}).unwrap();



	context.add_callback("__api_find_program", |name: String| {
		log::info!("finding {}...", name);
		JsValue::String(which::which(&name).unwrap().display().to_string())
	}).unwrap();

	context.add_callback("__api_find_program_or", |names: Vec<String>| -> Option<String> {
        for name in names {
            log::info!("trying to find {}...", name);
            if let Ok(path) = which::which(&name) {
                return Some(path.display().to_string());
            }
        }
        None
    }).unwrap();

	context.add_callback("__api_glob", |pattern: String| {
		match glob::glob(&pattern) {
			Ok(paths) => {
				let js_paths: Vec<JsValue> = paths
					.filter_map(Result::ok)
					.map(|path| {
						let path_str = path.to_string_lossy().into_owned();
						JsValue::String(path_str)
					})
					.collect();

				JsValue::Array(js_paths)
			}
			Err(_) => {
				log::warn!("glob {} resulted in no items", pattern);
				JsValue::Array(vec![])
			}
		}
	}).unwrap();

	context.add_callback("__api_executable", |name: String, opts: HashMap<String, JsValue>| {
		build_target_obj(name, opts, "executable")
	}).unwrap();

	context.add_callback("__api_library", |name: String, opts: HashMap<String, JsValue>| {
		build_target_obj(name, opts, "library")
	}).unwrap();

	context.add_callback("__api_msg", |txt: String| {
		log::info!("{}", txt);
		JsValue::String(txt)
	}).unwrap();

	context.eval(r#"
		globalThis.b = {
			toolchain: (info) => __api_toolchain(info),
			dependency: (name) => __api_dependency(name),
			glob: (pattern) => __api_glob(pattern),
			msg: (txt) => __api_msg(txt),
			executable: (name, options) => __api_executable(name, options),
			library: (name, options) => __api_library(name, options),
			findProgram: (name) => __api_find_program(name),
			findProgramOr: (options) => __api_find_program_or(options),
			systemLib: (name) => __api_system_lib(name),
			package: (pkg_name, pkg_lib) => __api_package(pkg_name, pkg_lib),
			option: (name, def) => __api_option(name, def),
			
			toolchains: {
				clang: (b) => {
					return b.toolchain({
				        cc: b.findProgram("clang"),
				        cxx: b.findProgram("clang++"),
				        ld: b.findProgramOr(["mold", "ld.lld", "ld"]),
				        ar: b.findProgram("ar")
				    });
				}
			}
		};
	"#).unwrap();
}

fn build_target_obj(name: String, opts: HashMap<String, JsValue>, kind: &str) -> JsValue {
	let _sources = extract_sources(&opts);
	let _collected_deps = extract_dependencies(&opts);

	let mut res_obj = HashMap::new();
	res_obj.insert("name".to_string(), JsValue::String(name.clone()));
	res_obj.insert("kind".to_string(), JsValue::String(kind.to_string()));

	if kind == "library" {
		let is_static = match opts.get("isStatic") {
		    Some(JsValue::Bool(b)) => *b,
		    _ => true,
		};
		res_obj.insert("isStatic".to_string(), JsValue::Bool(is_static));
		
		let mut cflags = String::new();
		if let Some(JsValue::Array(include_dirs)) = opts.get("includeDirectories") {
		    let formatted_dirs: Vec<String> = include_dirs
		        .iter()
		        .filter_map(|val| match val {
		            JsValue::String(s) => Some(format!("-I$(SOURCES_ROOT)/{}", s)),
		            _ => None,
		        })
		        .collect();
		    cflags = formatted_dirs.join(" ");
		}
		
		let mut ldflags = format!("-L$(SOURCES_ROOT)/.basalt -l{}", name);
		
		if !is_static {
		    ldflags.push_str("");
		}
		
		res_obj.insert("cflags".to_string(), JsValue::String(cflags));
		res_obj.insert("ldflags".to_string(), JsValue::String(ldflags));
	}

	res_obj.insert("options".to_string(), JsValue::Object(opts));
	JsValue::Object(res_obj)
}

fn extract_sources(opts: &HashMap<String, JsValue>) -> Vec<String> {
	if let Some(JsValue::Array(items)) = opts.get("sources") {
		items
			.iter()
			.filter_map(|item| match item {
				JsValue::String(s) => Some(s.clone()),
				_ => None,
			})
			.collect()
	} else {
		Vec::new()
	}
}

fn extract_dependencies(opts: &HashMap<String, JsValue>) -> Vec<Dependency> {
	let mut collected_deps = Vec::new();
	if let Some(JsValue::Array(deps)) = opts.get("dependencies") {
		for dep in deps {
			if let JsValue::Object(dep_fields) = dep {
				let cflags = match dep_fields.get("cflags") {
					Some(JsValue::String(s)) => s.clone(),
					_ => String::new(),
				};
				let ldflags = match dep_fields.get("ldflags") {
					Some(JsValue::String(s)) => s.clone(),
					_ => String::new(),
				};

				let local_name = match (dep_fields.get("kind"), dep_fields.get("name")) {
					(Some(JsValue::String(k)), Some(JsValue::String(n))) if k == "library" => {
						Some(n.clone())
					}
					_ => None,
				};

				collected_deps.push(Dependency::new(cflags, ldflags, local_name));
			}
		}
	}
	collected_deps
}
