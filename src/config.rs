use std::collections::HashMap;
use quick_js::JsValue;

#[derive(Debug)]
pub struct Dependency {
    pub ldflags: String,
    pub cflags: String,
}

impl Dependency {
    pub fn new(cflags: String, ldflags: String) -> Self {
        Self { cflags, ldflags }
    }
}

#[derive(Debug)]
pub struct Executable {
    pub name: String,
    pub sources: Vec<String>,
    pub dependencies: Vec<Dependency>,
    pub cc: String,
    pub cxx: String,
    pub ld: String,
    pub ar: String,
}

fn extract_string(map: &HashMap<String, JsValue>, key: &str) -> String {
    match map.get(key) {
        Some(JsValue::String(s)) => s.clone(),
        _ => String::new(),
    }
}

pub fn parse_executables(result: JsValue) -> Vec<Executable> {
    let mut executables = Vec::new();

    if let JsValue::Array(targets) = result {
        for target in targets {
            if let JsValue::Object(target_obj) = target {
                let mut exec_name = String::new();
                if let Some(JsValue::String(name)) = target_obj.get("name") {
                    exec_name = name.clone();
                }

                let mut sources = Vec::new();
                let mut collected_deps = Vec::new();
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

                    if let Some(JsValue::Array(deps)) = opts.get("dependencies") {
                        for dep in deps {
                            if let JsValue::Object(dep_fields) = dep {
                                let cflags = extract_string(dep_fields, "cflags");
                                let ldflags = extract_string(dep_fields, "ldflags");
                                collected_deps.push(Dependency::new(cflags, ldflags));
                            }
                        }
                    }

                    if let Some(JsValue::Object(toolchain_fields)) = opts.get("toolchain") {
                        cc = extract_string(toolchain_fields, "cc");
                        cxx = extract_string(toolchain_fields, "cxx");
                        ld = extract_string(toolchain_fields, "ld");
                        ar = extract_string(toolchain_fields, "ar");
                    }
                }

                executables.push(Executable {
                    name: exec_name,
                    sources,
                    dependencies: collected_deps,
                    cc,
                    cxx,
                    ld,
                    ar,
                });
            }
        }
    }

    executables
}
