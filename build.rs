use std::{
	env,
	path::{Path, PathBuf},
};

macro_rules! link_lib {
	( $fmt:literal, $($x:tt),* ) => {
		println!("cargo:rustc-link-lib={}", format!($fmt, $($x),* ));
	}
}

#[cfg(debug_assertions)]
const VERSION: &str = "Debug";

#[cfg(not(debug_assertions))]
const VERSION: &str = "Release";

fn main() {
	let out_dir = env::var("OUT_DIR").expect("Couldn't get OUT_DIR");
	let out_dir = Path::new(&out_dir);

	let bindings = bindgen::Builder::default()
		.header("flex.hpp")
		.allowlist_function("NvFlex.*")
		.allowlist_type("NvFlex.*")
		.allowlist_var("NvFlex.*")
		.allowlist_var("NV_FLEX_.*");

	#[cfg(feature = "Ext")]
	let bindings = bindings.clang_arg("-DUSE_NV_EXT");

	let bindings = bindings
		.generate()
		.expect("Couldn't generate bindings!");

	bindings
		.write_to_file(out_dir.join("bindings.rs"))
		.expect("Couldn't write bindings!");

	// Disable default features if you don't want to link
	link();
}

#[cfg(all(target_os = "windows"))]
fn link() {
	let ptr_width = env::var("CARGO_CFG_TARGET_POINTER_WIDTH")
		.expect("Couldn't get CARGO_CFG_TARGET_POINTER_WIDTH");

	let end = if ptr_width == "64" { "64" } else { "86" };

	let path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
		.join("FleX")
		.join("lib")
		.join(format!("win{}", end));

	println!("cargo:rustc-link-search=native={}", path.display());

	#[cfg(feature = "D3D")]
	{
		link_lib!("NvFlex{}D3D_x{}", VERSION, end);
		#[cfg(feature = "Ext")]
		link_lib!("NvFlexExt{}D3D_x{}", VERSION, end);
	}

	#[cfg(feature = "CUDA")]
	{
		link_lib!("NvFlex{}CUDA_x{}", VERSION, end);

		#[cfg(feature = "Ext")]
		link_lib!("NvFlexExt{}CUDA_x{}", VERSION, end);
	}
}

#[cfg(all(target_os = "linux"))]
fn link() {
	let ptr_width = env::var("CARGO_CFG_TARGET_POINTER_WIDTH")
		.expect("Couldn't get CARGO_CFG_TARGET_POINTER_WIDTH");

	assert!(ptr_width == "64", "Only 64-bit Linux is supported!");

	let path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
		.join("FleX")
		.join("lib")
		.join("linux64");

	println!("cargo:rustc-link-search=native={}", path.display());

	#[cfg(feature = "D3D")]
	panic!("D3D is not supported on Linux, enable the CUDA feature");

	#[cfg(feature = "CUDA")]
	{
		link_lib!("NvFlex{}CUDA_x64", VERSION);

		#[cfg(feature = "Ext")]
		link_lib!("NvFlexExt{}CUDA_x64", VERSION);
	}
}

#[cfg(all(target_os = "android"))]
fn link() {
	let path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
		.join("FleX")
		.join("lib")
		.join("android");

	println!("cargo:rustc-link-search=native={}", path.display());

	#[cfg(feature = "D3D")]
	panic!("D3D is not supported on Android, use the CUDA feature!");

	#[cfg(feature = "CUDA")]
	{
		link_lib!("libNvFlex{}CUDA_aarch64", VERSION);

		#[cfg(feature = "Ext")]
		link_lib!("libNvFlexExt{}CUDA_aarch64", VERSION);
	}
}
