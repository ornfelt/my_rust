use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Tell Rust where to find the SDL2.lib or equivalent
    println!("cargo:rustc-link-search=native=C:/local/SDL2-devel-2.30.6-VC/SDL2-2.30.6/lib/x64");
    println!("cargo:rustc-link-lib=SDL2");

    // Get the path to the target directory (where the build artifacts are)
    let out_dir = env::var("OUT_DIR").unwrap();
    let target_dir = PathBuf::from(out_dir).ancestors().nth(3).unwrap().to_path_buf();

    // Path to the SDL2.dll in your local development environment
    let dll_src = PathBuf::from("C:/local/SDL2-devel-2.30.6-VC/SDL2-2.30.6/lib/x64/SDL2.dll");

    // Destination path: copy to the target directory
    let dll_dest = target_dir.join("SDL2.dll");

    // Copy the SDL2.dll to the target directory
    fs::copy(&dll_src, &dll_dest).expect("Failed to copy SDL2.dll to the target directory");
}
