use std::fs::DirEntry;

fn compile_folder(entry: DirEntry) -> std::io::Result<()> {
    if entry.metadata()?.is_dir() {
        let dir = std::fs::read_dir(entry.path())?;
        for entry in dir {
            compile_folder(entry?)?;
        }
    } else {
        slint_build::compile(entry.path())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    }
    Ok(())
}

fn main() {
    let config = slint_build::CompilerConfiguration::new().with_library_paths(
        std::collections::HashMap::from([(
            "material".to_string(),
            std::path::Path::new(&std::env::var_os("CARGO_MANIFEST_DIR").unwrap())
                .join("material-1.0/material.slint"),
        )]),
    );
    slint_build::compile_with_config("ui/main.slint", config).unwrap();
}
