use std::env;
use std::path::PathBuf;

fn main() {
    // Get the directory where the build script is located
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_path = PathBuf::from(&manifest_dir);
    let bridge_dir = manifest_path.join("src/bridge");

    println!("cargo:rerun-if-changed=src/bridge/geodesk_bridge.h");
    println!("cargo:rerun-if-changed=src/bridge/geodesk_bridge.cpp");
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=CMakeLists.txt");

    // Use CMake to build GeoDESK and the bridge
    // This automatically fetches GeoDESK via FetchContent
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(&out_dir);

    println!("cargo:warning=Building GeoDESK and bridge with CMake...");

    let dst = cmake::Config::new(&manifest_path)
        .define("CMAKE_BUILD_TYPE", "Release")
        .define("BUILD_SHARED_LIBS", "OFF")
        .build();

    println!("cargo:warning=CMake build completed at: {}", dst.display());

    // Add library search paths
    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-search=native={}/build", dst.display());
    println!("cargo:rustc-link-search=native={}/build/lib", dst.display());

    // Also check in the CMake build directory
    let cmake_build_dir = out_path.join("build");
    println!("cargo:rustc-link-search=native={}/lib", cmake_build_dir.display());
    println!("cargo:rustc-link-search=native={}", cmake_build_dir.display());

    // Link to GeoDESK (static library built by CMake)
    println!("cargo:rustc-link-lib=static=geodesk");

    // Get the GeoDESK include directory from CMake build
    let geodesk_include = cmake_build_dir.join("_deps/geodesk-src/include");

    // Build the cxx bridge with proper includes
    let mut build = cxx_build::bridge("src/lib.rs");
    build
        .file(bridge_dir.join("geodesk_bridge.cpp"))
        .flag_if_supported("-std=c++20")
        .flag_if_supported("/std:c++20") // MSVC
        .include(&bridge_dir);

    // Add GeoDESK include path if it exists
    if geodesk_include.exists() {
        build.include(&geodesk_include);
        println!("cargo:warning=Using GeoDESK headers from: {}", geodesk_include.display());
    } else {
        // Try to find it in the install prefix
        let alt_include = dst.join("include");
        if alt_include.exists() {
            build.include(&alt_include);
        }
    }

    build.compile("geodesk_bridge_cxx");

    // On Linux, link to standard C++ library and other dependencies
    #[cfg(target_os = "linux")]
    {
        println!("cargo:rustc-link-lib=dylib=stdc++");
        println!("cargo:rustc-link-lib=dylib=pthread");
        println!("cargo:rustc-link-lib=dylib=dl");
        println!("cargo:rustc-link-lib=dylib=m");
    }

    // On macOS, link to libc++
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-lib=dylib=c++");
        println!("cargo:rustc-link-lib=dylib=System");
    }

    // On Windows, the C++ standard library is linked automatically by MSVC
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-lib=dylib=ws2_32");
    }
}

