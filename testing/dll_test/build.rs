// build.rs
//fn main() {
//    println!("cargo:rustc-link-search=native=/home/jonas/Code2/C++/my_cplusplus/Navigation/Pathing/build");
//}

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    // Check OS
    if cfg!(target_os = "windows") {
        if let Ok(code_root_dir) = env::var("code_root_dir") {
            // Define source path
            let source_path = format!(
                "{}\\Code2\\C++\\my_cplusplus\\Navigation\\Pathing\\build\\Release\\Navigation.dll",
                code_root_dir
            );
            let link_path = format!(
                "{}\\Code2\\C++\\my_cplusplus\\Navigation\\Pathing\\build\\Release",
                code_root_dir
            );
            println!("cargo:rustc-link-search=native={}", link_path);

            // Define destination path in the project root
            let project_root_destination = Path::new("Navigation.dll");

            // Copy DLL to the project root
            if let Err(e) = fs::copy(&source_path, &project_root_destination) {
                eprintln!(
                    "Error: Failed to copy Navigation.dll from '{}' to '{}': {}",
                    source_path,
                    project_root_destination.display(),
                    e
                );
                return;
            } else {
                println!(
                    "Successfully copied Navigation.dll from '{}' to '{}'.",
                    source_path,
                    project_root_destination.display()
                );
            }

            // Ensure src dir exists
            let src_dir = Path::new("src");
            if !src_dir.exists() {
                if let Err(e) = fs::create_dir_all(&src_dir) {
                    eprintln!("Error: Failed to create 'src' directory: {}", e);
                    return;
                }
            }

            // Define destination path in the src dir
            let src_destination = src_dir.join("Navigation.dll");

            // Also copy DLL from project root to the src directory
            if let Err(e) = fs::copy(&project_root_destination, &src_destination) {
                eprintln!(
                    "Error: Failed to copy Navigation.dll from '{}' to '{}': {}",
                    project_root_destination.display(),
                    src_destination.display(),
                    e
                );
            } else {
                println!(
                    "Successfully copied Navigation.dll from '{}' to '{}'.",
                    project_root_destination.display(),
                    src_destination.display()
                );
            }
        } else {
            eprintln!("Error: The environment variable `code_root_dir` is not set.");
        }
    } else {
        // Non-Windows
        println!("cargo:rustc-link-search=native=/home/jonas/Code2/C++/my_cplusplus/Navigation/Pathing/build");
    }
}

