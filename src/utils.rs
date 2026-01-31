#![allow(dead_code)]

use std::sync::{Arc, Mutex};
use gtk::glib::{self, object::IsA};
use adw::prelude::AdwDialogExt;
use smol::io::AsyncBufReadExt;

use crate::{models::{self, Game, Settings}, ui::logs_dialog::LogsDialog};

pub fn command_exists(cmd: &str) -> bool {
    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(&format!("command -v {cmd} >/dev/null 2>&1"))
        .status()
        .unwrap();
    status.code() == Some(0)
}

pub fn run_with_logs<F, Fut>(
    parent: &impl IsA<gtk::Widget>,
    game: models::Game,
    settings: models::Settings,
    runner: F,
)
where
    F: FnOnce(Game, Settings, Box<dyn FnMut(String) + Send + Sync + 'static>) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    let logs_dialog = LogsDialog::new();
    logs_dialog.present(Some(parent));

    let (sender, receiver) = smol::channel::unbounded::<String>();

    glib::spawn_future_local(glib::clone!(
        #[weak] logs_dialog,
        async move {
            while let Ok(message) = receiver.recv().await {
                logs_dialog.append_line(message);
            }
        }
    ));

    smol::spawn(async move {
        runner(
            game,
            settings,
            Box::new(move |line| {
                let sender = sender.clone();
                let _ = smol::block_on(async move {
                    sender.send(line).await.ok();
                });
            }),
        )
        .await;
    })
    .detach();
}

pub async fn launch_game(game: &Game, settings: &Settings, callback: impl FnMut(String) + Send + Sync + 'static) {
    let exe_dir = game.exe_path.parent().unwrap();
    let exe_name = game.exe_path.file_name().unwrap();
    let wine_path = settings.proton_path.join("files").join("bin").join("wine");
    let umu_run_path = settings.umu_path.join("umu-run");
    let using_gamescope = game.gamescope_config.is_some();
    let inside_flatpak = std::env::var("FLATPAK_ID").is_ok();
    let mut command: smol::process::Command;

    let env_vars = vec![
        ("WINEPREFIX", game.wineprefix_path.to_string_lossy().into_owned()),
        ("PROTONPATH", settings.proton_path.to_string_lossy().into_owned()),
        ("GAME_NAME", game.name.clone()),
        ("WINEDEBUG", "-all".to_string()),
        ("DXVK_LOG_LEVEL", "debug".to_string()),
        ("PROTON_LOG", "1".to_string()),
        ("UMU_LOG", "debug".to_string()),
        ("WINEARCH", "win64".to_string()),
        ("WINE", wine_path.to_string_lossy().into_owned()),
        ("WINEESYNC", "0".to_string()),
        ("WINEFSYNC", "1".to_string()),
        ("WINE_FULLSCREEN_FSR", "1".to_string()),
        ("DXVK_NVAPIHACK", "0".to_string()),
        ("DXVK_ENABLE_NVAPI", "1".to_string()),
        ("WINEDLLOVERRIDES", "d3d10core,d3d11,d3d12,d3d12core,d3d8,d3d9,d3dcompiler_33,d3dcompiler_34,d3dcompiler_35,d3dcompiler_36,d3dcompiler_37,d3dcompiler_38,d3dcompiler_39,d3dcompiler_40,d3dcompiler_41,d3dcompiler_42,d3dcompiler_43,d3dcompiler_46,d3dcompiler_47,d3dx10,d3dx10_33,d3dx10_34,d3dx10_35,d3dx10_36,d3dx10_37,d3dx10_38,d3dx10_39,d3dx10_40,d3dx10_41,d3dx10_42,d3dx10_43,d3dx11_42,d3dx11_43,d3dx9_24,d3dx9_25,d3dx9_26,d3dx9_27,d3dx9_28,d3dx9_29,d3dx9_30,d3dx9_31,d3dx9_32,d3dx9_33,d3dx9_34,d3dx9_35,d3dx9_36,d3dx9_37,d3dx9_38,d3dx9_39,d3dx9_40,d3dx9_41,d3dx9_42,d3dx9_43,dxgi,nvapi,nvapi64,nvofapi64=n;winemenubuilder=".to_string()),
        ("WINE_LARGE_ADDRESS_AWARE", "1".to_string()),
        ("STORE", "none".to_string()),
        ("GAMEID", "umu-default".to_string()),
        ("PROTON_VERB", "run".to_string()),
    ];

    if inside_flatpak {
        command = smol::process::Command::new("flatpak-spawn");
        command.arg("--host");
        for (key, val) in &env_vars {
            command.arg(format!("--env={key}={val}"));
        }
        if using_gamescope {
            command.arg("gamescope");
        } else {
            command.arg(&umu_run_path);
        }
    } else {
        if using_gamescope {
            command = smol::process::Command::new("gamescope");
        } else {
            command = smol::process::Command::new(&umu_run_path);
        }
        for (key, val) in &env_vars {
            command.env(key, val);
        }
    }

    if let Some(config) = &game.gamescope_config {
        command
            .args([
                "-W", &config.output_width.to_string(),
                "-H", &config.output_height.to_string(),
                "-f", "--force-grab-cursor",
                "--", umu_run_path.to_str().unwrap_or_default(),
            ]);
    }

    command
        .arg(exe_name)
        .arg(&game.extra_args)
        .current_dir(exe_dir);

    spawn_with_streaming_capture(command, callback).await;
}

pub async fn launch_winecfg(game: &Game, settings: &Settings, callback: impl FnMut(String) + Send + Sync + 'static) {
    let umu_run_path = settings.umu_path.join("umu-run");

    let mut command = smol::process::Command::new(&umu_run_path);
    command
        .arg("winecfg")
        .env("WINEPREFIX", game.wineprefix_path.as_os_str())
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
                    "{}",
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
                    "{}",
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
