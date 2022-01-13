// See https://doc.rust-lang.org/cargo/reference/build-scripts.html for documentation.
// The main() function here is executed by the Rust build system before building the rest of the crate.

extern crate bindgen;

fn main() {
    // Tell cargo to tell rustc to link the system bzip2
    // shared library.
    //println!("cargo:rustc-link-lib=bz2");

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper/wrapper.h");

    #[cfg(feature="enable_garbage_collection")]
    {
        // Tell cargo to link with the libbronze static library
        println!("cargo:rustc-link-lib=static=bronze_gc");
        let mut libbronze_dir = PathBuf::from(&env::var("CARGO_MANIFEST_DIR").unwrap());
        libbronze_dir.pop(); // get parent dir
        libbronze_dir.push("libbronze");

        let mut libbronze_build_dir = libbronze_dir.clone();
        libbronze_build_dir.push("build");

        let lib_path_str = libbronze_build_dir.as_path().to_str().expect("libbronze path must be a valid string");
        
        println!("cargo:rustc-link-search=native={}", lib_path_str);

        // Mark dependencies on libbronze.
        let mut libbronze_path = libbronze_build_dir.clone();
        libbronze_path.push("libbronze.a");
        let libbronze_path_str = libbronze_path.to_str().expect("libbronze.a path must be a valid string");

        println!("cargo:rerun-if-changed=wrapper/wrapper.h");
        println!("cargo:rerun-if-changed={}", libbronze_path_str);

        // The bindgen::Builder is the main entry point
        // to bindgen, and lets you build up options for
        // the resulting bindings.
        let bindings = bindgen::Builder::default()
            // The input header we would like to generate
            // bindings for.
            .header("wrapper/wrapper.h")
            // Tell cargo to invalidate the built crate whenever any of the
            // included header files changed.
            .parse_callbacks(Box::new(bindgen::CargoCallbacks))
            // Finish the builder and generate the bindings.
            .generate()
            // Unwrap the Result and panic on failure.
            .expect("Unable to generate bindings");

        // Write the bindings to the $OUT_DIR/bindings.rs file.
        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        bindings
            .write_to_file(out_path.join("bindings.rs"))
            .expect("Couldn't write bindings!");
    }
}