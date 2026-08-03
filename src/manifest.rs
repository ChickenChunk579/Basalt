use std::collections::HashMap;
use quick_js::JsValue;

use crate::dependency::Dependency;
use crate::executable::Executable;
use crate::library::Library;

#[derive(Debug)]
pub enum Target {
	Executable(Executable),
	Library(Library),
}

struct CommonFields {
	name: String,
	sources: Vec<String>,
	dependencies: Vec<Dependency>,
	include_directories: Vec<String>,
	define: Vec<String>,
	cc: String,
	cxx: String,
	ld: String,
	ar: String,
}

pub fn parse_targets(result: JsValue) -> Vec<Target> {
	let mut targets = Vec::new();
	if let JsValue::Array(items) = result {
		for item in items {
			if let JsValue::Object(target_obj) = item {
				if let Some(target) = parse_target(target_obj) {
					targets.push(target);
				}
			}
		}
	}
	targets
}

fn parse_target(target_obj: HashMap<String, JsValue>) -> Option<Target> {
	let kind = match target_obj.get("kind") {
		Some(JsValue::String(k)) => k.clone(),
		_ => return None,
	};

	let common = parse_common(&target_obj);

	match kind.as_str() {
		"library" => {
			let is_static = match target_obj.get("isStatic") {
				Some(JsValue::Bool(b)) => *b,
				_ => true,
			};
			Some(Target::Library(Library {
				name: common.name,
				sources: common.sources,
				dependencies: common.dependencies,
				include_directories: common.include_directories,
				define: common.define,
				cc: common.cc,
				cxx: common.cxx,
				ld: common.ld,
				ar: common.ar,
				is_static,
			}))
		}
		_ => Some(Target::Executable(Executable {
			name: common.name,
			sources: common.sources,
			dependencies: common.dependencies,
			include_directories: common.include_directories,
			define: common.define,
			cc: common.cc,
			cxx: common.cxx,
			ld: common.ld,
			ar: common.ar,
		})),
	}
}

fn parse_common(target_obj: &HashMap<String, JsValue>) -> CommonFields {
	let mut name = String::new();
	if let Some(JsValue::String(n)) = target_obj.get("name") {
		name = n.clone();
	}

	let mut sources = Vec::new();
	let mut dependencies = Vec::new();
	let mut include_directories = Vec::new();
	let mut define = Vec::new();
	let mut cc = String::new();
	let mut cxx = String::new();
	let mut ld = String::new();
	let mut ar = String::new();

	if let Some(JsValue::Object(opts)) = target_obj.get("options") {
		if let Some(JsValue::Array(items)) = opts.get("sources") {
			sources = items
				.iter()
				.filter_map(|item| match item {
					JsValue::String(s) => Some(s.clone()),
					_ => None,
				})
				.collect();
		}
		if let Some(JsValue::Array(items)) = opts.get("includeDirectories") {
			include_directories = items
				.iter()
				.filter_map(|item| match item {
					JsValue::String(s) => Some(s.clone()),
					_ => None,
				})
				.collect();
		}
		if let Some(JsValue::Array(items)) = opts.get("define") {
			define = items
				.iter()
				.filter_map(|item| match item {
					JsValue::String(s) => Some(s.clone()),
					_ => None,
				})
				.collect();
		}

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

					dependencies.push(Dependency::new(cflags, ldflags, local_name));
				}
			}
		}

		if let Some(JsValue::Object(toolchain_fields)) = opts.get("toolchain") {
			let extract_string = |map: &HashMap<String, JsValue>, key: &str| -> String {
				match map.get(key) {
					Some(JsValue::String(s)) => s.clone(),
					_ => String::new(),
				}
			};
			cc = extract_string(toolchain_fields, "cc");
			cxx = extract_string(toolchain_fields, "cxx");
			ld = extract_string(toolchain_fields, "ld");
			ar = extract_string(toolchain_fields, "ar");
		}
	}

	CommonFields { name, sources, dependencies, include_directories, define, cc, cxx, ld, ar }
}
