fn main() {
    println!("cargo:warning=Running build script...");

    // Collect all .blp files
    let blp_files: Vec<_> = std::fs::read_dir("resources/ui")
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()? == "blp" {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    // Tell cargo to rerun if any .blp file changes
    for blp_file in &blp_files {
        println!("cargo:rerun-if-changed={}", blp_file.display());
    }

    println!("cargo:warning=Compiling {} blueprint files...", blp_files.len());

    // Compile all .blp files
    let mut cmd = std::process::Command::new("blueprint-compiler");
    cmd.arg("batch-compile")
        .arg("resources/ui")
        .arg("resources/ui")
        .args(&blp_files);

    let status = cmd.status().unwrap();
    assert!(status.success());

    println!("cargo:warning=Blueprint compilation complete");

    glib_build_tools::compile_resources(&["./resources"], "./resources/resources.xml", "compiled.gresource");
}
