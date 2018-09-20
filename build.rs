use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
	let cargo_manifest_dir = &env::var("CARGO_MANIFEST_DIR").unwrap();
	let current_directory = Path::new(&cargo_manifest_dir);

	println!("Cargo:current_directory: {:?}", current_directory);

	Command::new("git")
		.arg("submodule")
		.arg("update")
		.arg("--init")
		.arg("--recursive")
		.current_dir(&current_directory)
		.status()
		.unwrap();
}
