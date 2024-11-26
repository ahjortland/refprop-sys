use std::{env, path::PathBuf, process::Command};

use cmake::Config;
use git2::Repository;

fn main() {
    // Define the output directory where dependencies will be built.
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let lib_dir = out_dir.join("lib");
    let refprop_cmake_dir = out_dir.join("REFPROP-cmake");

    // Clone the REFPROP-cmake repository if it hasn't been cloned yet.
    if !refprop_cmake_dir.exists() {
        println!("Cloning REFPROP-cmake repository...");
        Repository::clone_recurse(
            "https://github.com/ahjortland/REFPROP-cmake.git",
            &refprop_cmake_dir,
        )
        .expect("REFPROP-cmake repository already cloned.");
    } else {
        println!("REFPROP-cmake repository already cloned.");
    }

    // Path the the Fortran source files.
    let fortran_dir = PathBuf::from(
        env::var("REFPROP_SRC_DIR").expect("REFPROP_SRC_DIR environment variable not set."),
    );

    // ======================
    // Compiler and Build Configuration
    // ======================
    #[cfg(target_os = "macos")]
    let fortran_compiler = env::var("CMAKE_FORTRAN_COMPILER")
        .unwrap_or_else(|_| "/opt/homebrew/bin/gfortran".to_string());

    let python_executable = env::var("PYTHON_EXECUTABLE")
        .unwrap_or_else(|_| "/Users/andrew/.pyenv/versions/3.12.5/bin/python".to_string());

    // Optional: Check for the presence of the Fortran compiler.
    let compiler_check = Command::new(&fortran_compiler).arg("--version").output();
    if compiler_check.is_err() || !compiler_check.unwrap().status.success() {
        panic!(
            "Fortran compiler not found at {:?}. Please install gfortran or update the path.",
            fortran_compiler
        );
    }

    let python_check = Command::new(&python_executable).arg("--version").output();
    if python_check.is_err() || !python_check.unwrap().status.success() {
        panic!(
            "Python executable not found at {:?}. Please install python or update the path.",
            python_executable
        );
    }

    let build_type = env::var("CMAKE_BUILD_TYPE").unwrap_or_else(|_| "Release".to_string());

    // Configure CMake with custom options.
    let mut cmake_config = Config::new(&refprop_cmake_dir);
    cmake_config
        .define("REFPROP_FORTRAN_PATH", &fortran_dir)
        .define("CMAKE_FORTRAN_COMPILER", &fortran_compiler)
        .define("CMAKE_BUILD_TYPE", &build_type)
        .define("PYTHON_EXECUTABLE", &python_executable);

    println!("Building REFPROP-cmake library with CMake...");

    // Build the library.
    cmake_config.build();

    // Link the installed library.
    println!(
        "cargo:rustc-link-search=native={}/{}",
        out_dir.display(),
        lib_dir.file_name().unwrap().to_str().unwrap()
    );
    println!("cargo:rustc-link-lib=dylib=refprop");

    // Instruct Cargo to rerun the build script if Fortran files or CMakeLists.txt change.
    println!("cargo:rerun-if-changed=REFPROP_FORTRAN/");
    println!(
        "cargo:rerun-if-changed={}/CMakeLists.txt",
        refprop_cmake_dir.display()
    );

    // ======================
    // Bindgen Integration
    // ======================
    // Path to the generated C headers after building REFPROP-cmake
    let headers_dir = out_dir.join("include");
    let header_file = headers_dir.join("REFPROP.h");

    if !header_file.exists() {
        panic!(
            "C header file not found at {}. Ensure that REFPROP-cmake generates C headers.",
            header_file.display()
        )
    }

    let bindings = bindgen::Builder::default()
        .header(header_file.to_str().unwrap())
        // Include the headers directory for include files
        .clang_arg(format!("-I{}", headers_dir.display()))
        // Adjust based on the naming conventions of REFPROP's API.
        .allowlist_function(".*")
        .allowlist_type(".*")
        .allowlist_var(".*")
        // Generate bindings for all symbols matching the patterns.
        .generate()
        .expect("Unable to generate bindings with bindgen.");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let bindings_out_path = out_dir.join("bindings.rs");
    bindings
        .write_to_file(&bindings_out_path)
        .expect("Couldn't write bindings.");

    // Inform Cargo to include the bindings.rs file via environment variable.
    println!(
        "cargo:rustc-env=BINDINGS_PATH={}",
        bindings_out_path.display()
    );
}
