fn main() {
    println!("cargo:rerun-if-changed=resources/ui");
    let status = std::process::Command::new("blueprint-compiler")
        .arg("batch-compile")
        .arg("resources/ui")
        .arg("resources/ui")
        .args([
            "resources/ui/window.blp",
            "resources/ui/grid_item.blp",
            "resources/ui/add_edit_game_page.blp"
        ])
        .status()
        .unwrap();
    assert!(status.success());

    glib_build_tools::compile_resources(&["./resources"], "./resources/resources.xml", "compiled.gresource");
}
