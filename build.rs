use cmake::Config;

fn main() {
	let dst = Config::new("renderer")
						.build();

	println!("cargo:rustc-link-search=native={}", dst.display());
	println!("cargo:rustc-link-lib=static=glad");
	println!("cargo:rustc-link-lib=static=renderer");
}