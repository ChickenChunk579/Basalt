use std::process::Command;
use std::fs;
use quick_js::{Context, JsValue};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Dependency {
	pub cflags: String,
	pub ldflags: String,
	pub local_name: Option<String>,
}

impl Dependency {
	pub fn new(cflags: String, ldflags: String, local_name: Option<String>) -> Dependency {
		Dependency {
			cflags,
			ldflags,
			local_name,
		}
	}
}

pub fn resolve_dependency_pkgconf(name: String) -> Option<Dependency> {
	let cflags_output = Command::new("pkg-config")
		.args(["--cflags", &name])
		.output();
	let ldflags_output = Command::new("pkg-config")
		.args(["--libs", &name])
		.output();

	match (cflags_output, ldflags_output) {
		(Ok(c_out), Ok(l_out)) if c_out.status.success() && l_out.status.success() => {
			let cflags = String::from_utf8_lossy(&c_out.stdout).trim().to_string();
			let ldflags = String::from_utf8_lossy(&l_out.stdout).trim().to_string();
			Some(Dependency::new(cflags, ldflags, None))
		}
		_ => None,
	}
}

pub fn resolve_bedrock_package(pkg_name: String, pkg_lib: String) -> Option<Dependency> {
    let package_dir = Path::new("bedrock-packages").join(&pkg_name);

    if !package_dir.is_dir() {
        return None;
    }

    let config_path = package_dir.join("basalt.config.js");
    if !config_path.exists() {
        return None;
    }

    let context = Context::new().expect("Failed to initialize QuickJS");
    crate::js_api::register(&context);

    let config = fs::read_to_string(&config_path).ok()?;

    context.eval(&config).ok()?;
    let value = context.eval("pkg(globalThis.b);").ok()?;

    if let JsValue::Object(map) = value {
        if let Some(JsValue::Object(target)) = map.get(&pkg_lib) {
            let sources_root = format!("$(SOURCES_ROOT)/bedrock-packages/{}", pkg_name);

            let cflags = match target.get("cflags") {
                Some(JsValue::String(s)) => {
                    s.replace("$(SOURCES_ROOT)", &sources_root)
                }
                _ => String::new(),
            };

            let ldflags = match target.get("ldflags") {
                Some(JsValue::String(s)) => {
                    s.replace("$(SOURCES_ROOT)", &sources_root)
                }
                _ => String::new(),
            };

            let dep = Dependency::new(cflags, ldflags, None);

            return Some(dep);
        }
    }

    None
}
