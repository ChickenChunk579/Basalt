use std::process::Command;

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
