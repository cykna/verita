use std::fs::DirEntry;
use std::path::PathBuf;
use std::collections::HashMap;

fn main() {
    println!("cargo:warning=LUCIDE ICON PATH: {:?}", lucide_slint::lib());
    let library = HashMap::from([
        ("lucide".to_string(), PathBuf::from(lucide_slint::lib())),
        ("material".to_string(), std::path::Path::new(&std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("material-1.0/material.slint")),
    ]);
    let config = slint_build::CompilerConfiguration::new().with_library_paths(library);
    slint_build::compile_with_config("ui/main.slint", config).unwrap();
}
