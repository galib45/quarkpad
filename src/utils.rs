use std::process::Command;

use crate::models::{Game, Settings};

pub fn launch_game(game: &Game, settings: &Settings) {
    let exe_dir = game.exe_path.parent().unwrap();
    let exe_name = game.exe_path.file_name().unwrap();
    let wine_path = settings.proton_path.join("files").join("bin").join("wine");
    let umu_run_path = settings.umu_path.join("umu-run");

    Command::new("gamescope")
        .arg("-W").arg("1920")
        .arg("-H").arg("1080")
        .arg("-f")
        .arg("--force-grab-cursor")
        .arg("--")
        .arg(umu_run_path)
        .arg(exe_name)
        .current_dir(exe_dir)
        .env("WINEPREFIX", game.wineprefix.as_os_str())
        .env("PROTONPATH", settings.proton_path.as_os_str())
        .env("GAME_NAME", game.name.as_str())
        .env("WINEDEBUG", "-all")
        .env("DXVK_LOG_LEVEL", "debug")
        .env("PROTON_LOG", "1")
        .env("UMU_LOG", "debug")
        .env("WINEARCH", "win64")
        .env("WINE", wine_path)
        .env("WINEESYNC", "0")
        .env("WINEFSYNC", "1")
        .env("WINE_FULLSCREEN_FSR", "1")
        .env("DXVK_NVAPIHACK", "0")
        .env("DXVK_ENABLE_NVAPI", "1")
        .env(
            "WINEDLLOVERRIDES",
            "d3d10core,d3d11,d3d12,d3d12core,d3d8,d3d9,\
             d3dcompiler_33,d3dcompiler_34,d3dcompiler_35,d3dcompiler_36,\
             d3dcompiler_37,d3dcompiler_38,d3dcompiler_39,d3dcompiler_40,\
             d3dcompiler_41,d3dcompiler_42,d3dcompiler_43,d3dcompiler_46,\
             d3dcompiler_47,d3dx10,d3dx10_33,d3dx10_34,d3dx10_35,d3dx10_36,\
             d3dx10_37,d3dx10_38,d3dx10_39,d3dx10_40,d3dx10_41,d3dx10_42,\
             d3dx10_43,d3dx11_42,d3dx11_43,d3dx9_24,d3dx9_25,d3dx9_26,\
             d3dx9_27,d3dx9_28,d3dx9_29,d3dx9_30,d3dx9_31,d3dx9_32,\
             d3dx9_33,d3dx9_34,d3dx9_35,d3dx9_36,d3dx9_37,d3dx9_38,\
             d3dx9_39,d3dx9_40,d3dx9_41,d3dx9_42,d3dx9_43,\
             dxgi,nvapi,nvapi64,nvofapi64=n;winemenubuilder=",
        )
        .env("WINE_LARGE_ADDRESS_AWARE", "1")
        .env("STORE", "none")
        .env("GAMEID", "umu-default")
        .env("PROTON_VERB", "run")
        .spawn().unwrap();
}
