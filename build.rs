use std::path::PathBuf;

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");
    let lib_dir = PathBuf::from(manifest_dir).join("libs").join("myteams");
    let lib_dir = lib_dir
        .canonicalize()
        .expect("libs/myteams directory must exist at build time");
    let lib_dir = lib_dir
        .to_str()
        .expect("libs/myteams path contains non UTF-8 characters");

    // Re-run linking setup if the shared library changes.
    println!("cargo:rerun-if-changed=libs/myteams/libmyteams.so");

    // Allow the linker to find libmyteams.so.
    println!("cargo:rustc-link-search=native={}", lib_dir);

    for bin in ["myteams_server", "myteams_cli"] {
        // Force the dynamic dependency even before any symbols are used.
        println!("cargo:rustc-link-arg-bin={bin}=-Wl,--no-as-needed");
        println!("cargo:rustc-link-arg-bin={bin}=-lmyteams");
        println!("cargo:rustc-link-arg-bin={bin}=-Wl,--as-needed");

        // Embed runtime lookup paths so no env var or launcher patching is needed.
        // println!("cargo:rustc-link-arg-bin={bin}=-Wl,-rpath,$ORIGIN/../../libs/myteams");
        // println!("cargo:rustc-link-arg-bin={bin}=-Wl,-rpath,$ORIGIN/libs/myteams");
    }
}
