use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let locales_dir = Path::new(&out_dir).join("locales");

    // 1. Compile .po files to .mo files
    let po_dir = Path::new("po");
    if po_dir.exists() {
        println!("cargo:rerun-if-changed=po");

        for entry in fs::read_dir(po_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("po") {
                let lang = path.file_stem().unwrap().to_str().unwrap();

                // Expected output path: target/locales/{lang}/LC_MESSAGES/mdgreet.mo
                let mo_dir = locales_dir.join(lang).join("LC_MESSAGES");
                fs::create_dir_all(&mo_dir).unwrap();
                let mo_path = mo_dir.join("mdgreet.mo");

                let status = Command::new("msgfmt")
                    .arg("-o")
                    .arg(&mo_path)
                    .arg(&path)
                    .status();

                match status {
                    Ok(s) if s.success() => {
                        println!("Compiled translation for {}", lang);
                    }
                    _ => {
                        println!(
                            "cargo:warning=Failed to compile translation for {}: msgfmt is missing or failed.",
                            lang
                        );
                    }
                }
            }
        }
    }

    // Export the locales directory to the compiler for use in dev mode
    println!("cargo:rustc-env=LOCALES_DIR_DEV={}", locales_dir.display());

    // 2. Compile Slint UI
    let config = slint_build::CompilerConfiguration::new().with_library_paths(
        std::collections::HashMap::from([(
            "material".to_string(),
            std::path::Path::new(&std::env::var_os("CARGO_MANIFEST_DIR").unwrap())
                .join("material-1.0/material.slint"),
        )]),
    );

    // Slint automatically enables gettext if the feature is enabled in Cargo.toml
    slint_build::compile_with_config("ui/main.slint", config).unwrap();
}
