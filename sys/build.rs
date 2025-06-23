use bindgen;
use std::env;
use std::path::Path;
use walkdir::WalkDir;

fn main() {
    println!("cargo:rerun-if-changed=src/wrapper.h");
    println!("cargo:rerun-if-changed=build.rs");

    let link_search_path = "cargo:rustc-link-search=native";
    let link_lib = "cargo:rustc-link-lib=static";

    let dkp = env::var("DEVKITPRO").expect("Please provided DEVKITPRO via env variables");
    let ppc = env::var("DEVKITPPC").expect("Please provided DEVKITPPC via env variables");

    println!("{link_search_path}={dkp}/wups/lib/");
    println!("{link_lib}=wups");

    // let blocked = vec!["xxx"];

    let headers: Vec<String> = WalkDir::new(Path::new(&format!("{dkp}/wups/include")))
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|entry| entry.path().is_file())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .map_or(false, |ext| ext.to_str() == Some("h"))
        })
        // .filter(|entry| {
        //     // Check if the path is NOT blocked
        //     let s = entry.path().display().to_string();
        //     !blocked.iter().any(|p| s.contains(p))
        // })
        .map(|entry| entry.path().display().to_string())
        .collect();

    let bindings = bindgen::Builder::default()
        .use_core()
        .headers(headers)
        .emit_builtins()
        .generate_cstr(true)
        .generate_comments(false)
        .default_enum_style(bindgen::EnumVariation::ModuleConsts)
        .prepend_enum_name(false)
        .layout_tests(false)
        .derive_default(true)
        .merge_extern_blocks(true)
        .wrap_unsafe_ops(true)
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
