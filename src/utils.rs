use std::sync::{Arc, Mutex};

use smol::io::AsyncBufReadExt;

use crate::{models::{Game, Settings}};

pub async fn launch_game(game: &Game, settings: &Settings, callback: impl FnMut(String) + Send + Sync + 'static) {
    let exe_dir = game.exe_path.parent().unwrap();
    let exe_name = game.exe_path.file_name().unwrap();
    let wine_path = settings.proton_path.join("files").join("bin").join("wine");
    let umu_run_path = settings.umu_path.join("umu-run");

    let mut command = if game.use_gamescope {
        smol::process::Command::new("gamescope")
    } else {
        smol::process::Command::new(&umu_run_path)
    };

    if game.use_gamescope {
        command
        .args([
            "-W", &game.gamescope_width.to_string(),
            "-H", &game.gamescope_height.to_string(),
            "-f", "--force-grab-cursor",
            "--", umu_run_path.to_str().unwrap_or_default(),
        ]);
    }

    command
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
        .env("PROTON_VERB", "run");

    spawn_with_streaming_capture(command, callback).await;
}

pub async fn launch_winecfg(game: &Game, settings: &Settings, callback: impl FnMut(String) + Send + Sync + 'static) {
    let umu_run_path = settings.umu_path.join("umu-run");

    let mut command = smol::process::Command::new(&umu_run_path);
    command
        .arg("winecfg")
        .env("WINEPREFIX", game.wineprefix.as_os_str())
        .env("PROTONPATH", settings.proton_path.as_os_str())
        .env("PROTON_LOG", "1")
        .env("WINEARCH", "win64");
    spawn_with_streaming_capture(command, callback).await;
}

async fn spawn_with_streaming_capture(
    mut command: smol::process::Command,
    on_line_callback: impl FnMut(String) + Send + Sync + 'static
) {
    command
        .stdout(smol::process::Stdio::piped())
        .stderr(smol::process::Stdio::piped());

    let mut child = command.spawn().unwrap();
    let on_line_cb_arc_mutex = Arc::new(Mutex::new(on_line_callback));

    let callback = on_line_cb_arc_mutex.clone();
    callback.lock().unwrap()(format!("[{}]", chrono::Local::now().format("%Y-%m-%d %I:%M:%S %p")));
    callback.lock().unwrap()("  RUNNING\n".into());
    callback.lock().unwrap()(format!("{:?}\n\n", command));

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let callback = on_line_cb_arc_mutex.clone();
    let stdout_task = smol::spawn(async move {
        let mut reader = smol::io::BufReader::new(stdout);
        let mut line = String::new();
        while reader.read_line(&mut line).await.unwrap() > 0 {
            callback.lock().unwrap()(
                format!(
                    "[STDOUT] {}",
                    String::from_utf8(strip_ansi_escapes::strip(&line)).unwrap()
                )
            );
            line.clear();
        }
    });

    let callback = on_line_cb_arc_mutex.clone();
    let stderr_task = smol::spawn(async move {
        let mut reader = smol::io::BufReader::new(stderr);
        let mut line = String::new();
        while reader.read_line(&mut line).await.unwrap() > 0 {
            callback.lock().unwrap()(
                format!(
                    "[STDERR] {}",
                    String::from_utf8(strip_ansi_escapes::strip(&line)).unwrap()
                )
            );
            line.clear();
        }
    });

    let status = child.status().await.unwrap();

    stdout_task.await;
    stderr_task.await;

    let callback = on_line_cb_arc_mutex.clone();
    if let Some(code) = status.code() {
        callback.lock().unwrap()("-------------------\n".into());
        callback.lock().unwrap()(format!("PROCESS EXITED WITH STATUS CODE {}\n", code));
        callback.lock().unwrap()("-------------------\n\n\n".into());
    }
}
