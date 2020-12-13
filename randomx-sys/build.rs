use std::{env, path::PathBuf};

fn main() {
    let config_src = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("mc_randomx")
        .join("MerosConfiguration")
        .join("configuration.h");
    let config_dst = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("RandomX")
        .join("src")
        .join("configuration.h");
    if !config_src.exists() || !config_dst.exists() {
        panic!("RandomX configuration doesn't exist. Did you checkout the git submodules?")
    }
    std::fs::copy(config_src, config_dst).unwrap();

    let dst = cmake::Config::new("RandomX")
        .define("ARCH", "native")
        .build_target("randomx")
        .profile("Release")
        .build();

    println!("cargo:rustc-link-search=native={}/build", dst.display());
    println!("cargo:rustc-link-lib=static=randomx");
    println!("cargo:rustc-link-lib=stdc++");
    println!("cargo:rerun-if-changed=RandomX/src/randomx.h");
    println!("cargo:rerun-if-changed=mc_randomx/MerosConfiguration/configuration.h");

    let bindings = bindgen::Builder::default()
        .header("RandomX/src/randomx.h")
        .whitelist_function("randomx_.*")
        .whitelist_type("randomx_.*")
        .whitelist_var("randomx_.*")
        .whitelist_var("RANDOMX_.*")
        .size_t_is_usize(true)
        .prepend_enum_name(false)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Failed to generate RandomX bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Failed to write RandomX bindings");
}
