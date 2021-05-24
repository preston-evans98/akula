use std::{env, path::PathBuf};

fn main() {
    let mut silkworm = PathBuf::from(&env::var("CARGO_MANIFEST_DIR").unwrap());
    silkworm.push("silkworm");
    silkworm.push("tg_api");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let bindings = bindgen::Builder::default()
        .header(silkworm.join("silkworm_tg_api.h").to_string_lossy())
        .detect_include_paths(true)
        .size_t_is_usize(true)
        .layout_tests(false)
        .prepend_enum_name(false)
        .generate_comments(false)
        .disable_header_comment()
        .rustfmt_bindings(true)
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    let dst = cmake::build("silkworm");

    println!("cargo:rustc-link-search=native={}", dst.display());
    println!("cargo:rustc-link-lib=silkworm_tg_api");
}
