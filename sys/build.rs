use std::env;

extern crate bindgen;

fn main() {
    println!("cargo:rerun-if-changed=src/wrapper.h");
    println!("cargo:rerun-if-changed=build.rs");

    let link_search_path = "cargo:rustc-link-search=native";
    let link_lib = "cargo:rustc-link-lib=static";

    let dkp = env::var("DEVKITPRO").expect("Please provided DEVKITPRO via env variables");
    let ppc = env::var("DEVKITPPC").expect("Please provided DEVKITPPC via env variables");

    println!("{link_search_path}={dkp}/wups/lib/");
    println!("{link_lib}=wups");

    let bindings = bindgen::Builder::default()
        .use_core()
        .header("src/wrapper.h")
        .emit_builtins()
        .generate_cstr(true)
        .generate_comments(false)
        .default_enum_style(bindgen::EnumVariation::ModuleConsts)
        .prepend_enum_name(false)
        .layout_tests(false)
        .derive_default(true)
        .merge_extern_blocks(true)
        .clang_args(vec![
            "--target=powerpc-none-eabi",
            &format!("--sysroot={ppc}/powerpc-eabi"),
            "-m32",
            "-mfloat-abi=hard",
            &format!("-I{dkp}/wups/include"),
            &format!("-I{dkp}/wut/include"),
            &format!("-I{ppc}/powerpc-eabi/include"),
        ])
        .allowlist_file(".*/wups/include/.*.h")
        .raw_line("#![allow(non_upper_case_globals)]")
        .raw_line("#![allow(non_camel_case_types)]")
        .raw_line("#![allow(non_snake_case)]")
        .raw_line("unsafe impl Sync for wups_loader_hook_t {}")
        .raw_line("unsafe impl Sync for wups_loader_entry_t {}")
        .generate()
        .expect("Unable to generate bindings");

    let out = std::path::PathBuf::from("./src/bindings.rs");
    bindings
        .write_to_file(&out)
        .expect("Unable to write bindings to file");
}
