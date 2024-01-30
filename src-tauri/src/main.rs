#![feature(lazy_cell)]
// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::LazyLock;

use directories::ProjectDirs;
use reqwest::Url;
use rodio::{Decoder, OutputStream, Sink};
use tauri::{
    api::notification::Notification, AppHandle, CustomMenuItem, Manager, SystemTray,
    SystemTrayEvent, SystemTrayMenu, WindowBuilder,
};
use tokio::time;

static PROJECT_IDENTIFIER: &'static str = "fyi.angelo.hydrate-reminder";
static PROJECT_DIR: LazyLock<ProjectDirs> =
    LazyLock::new(|| ProjectDirs::from("fyi", "angelo", "hydrate-reminder").unwrap());

static NOTIFICATION_AUDIO: LazyLock<&'static [u8]> =
    LazyLock::new(|| include_bytes!("../assets/notif.mp3"));
static DRINK_AUDIO: LazyLock<&'static [u8]> =
    LazyLock::new(|| include_bytes!("../assets/gulp.mp3"));

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn spawn_main_window(app: &AppHandle) {
    if let Some(main_window) = app.get_window("main") {
        main_window
            .set_focus()
            .expect("Unable to focus main window!")
    } else {
        WindowBuilder::new(app, "main", tauri::WindowUrl::App("index.html".into()))
            .hidden_title(true)
            .resizable(false)
            .closable(true)
            .build()
            .expect("Unable to create a new window!");
    }

    // window.on_window_event(|e| {
    //     match e {
    //         tauri::WindowEvent::Focused(false) => {
    //             window.close().expect("Failed to close window!");
    //         }
    //         _ => {}
    //     }

    //     println!("Window event: {:?}", e);
    // });
}

#[tauri::command]
fn create_drink_notification(app: AppHandle) {
    tokio::spawn(async move {
        let audio_buffer = std::io::Cursor::new(*NOTIFICATION_AUDIO);
        let audio = Decoder::new(audio_buffer).unwrap();

        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        sink.append(audio);
        Notification::new(&app.config().tauri.bundle.identifier)
            .title("Time to drink!")
            .body("It's been 1 hour since your last drink, time to drink again!")
            .show()
            .expect("Unable to create drink notification!");

        sink.sleep_until_end();
    });
}

fn play_drink_sound() {
    tokio::spawn(async move {
        let audio_buffer = std::io::Cursor::new(*DRINK_AUDIO);
        let audio = Decoder::new(audio_buffer).unwrap();

        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        sink.append(audio);
        sink.sleep_until_end()
    });
}

fn handle_tray_event(app: &AppHandle, event: SystemTrayEvent) {
    match event {
        tauri::SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
            "drink" => play_drink_sound(),
            "open-settings" => spawn_main_window(app),

            "quit" => app.exit(0),

            _ => {
                println!("Unknown menu item clicked: {}", id);
            }
        },
        _ => {}
    }
}

#[tauri::command]
fn handle_oauth_flow(app: AppHandle) {
    if let Some(window) = app.get_window("oauth") {
        window.set_focus().expect("Unable to focus oauth window!")
    } else {
        let qs = querystring::stringify(vec![
            (
                "client_id",
                "359154028055-42ip89g4r9m78pgoug1rpropgmbpgfa9.apps.googleusercontent.com",
            ),
            (
                "scope",
                "https://www.googleapis.com/auth/fitness.nutrition.write",
            ),
            ("prompt", "consent"),
            ("response_type", "code"),
            ("redirect_uri", "http://localhost:11132"),
            ("access_type", "online"),
        ]);
        let url =
            Url::parse(format!("https://accounts.google.com/o/oauth2/v2/auth?{}", qs).as_str())
                .unwrap();

        WindowBuilder::new(&app, "oauth", tauri::WindowUrl::External(url))
            .hidden_title(true)
            .resizable(false)
            .closable(true)
            .build()
            .expect("Unable to create a new window!");

        // Spawn OAuth server
        tokio::task::spawn(oauth_server_cb(app.app_handle()));
    }
}

#[tokio::main]
async fn main() {
    let tray_menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("drink", "🥛 Drink"))
        .add_item(CustomMenuItem::new("open-settings", "Open Settings"))
        .add_native_item(tauri::SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("quit", "Quit"));

    let tray = SystemTray::new().with_menu(tray_menu);

    let mut app = tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .system_tray(tray)
        .on_system_tray_event(handle_tray_event)
        .invoke_handler(tauri::generate_handler![
            create_drink_notification,
            greet,
            handle_oauth_flow
        ])
        .build(tauri::generate_context!())
        .expect("Error while running tauri application");

    app.set_activation_policy(tauri::ActivationPolicy::Accessory);

    tokio::task::spawn(notification_task(app.app_handle()));

    app.run(|_, e| match e {
        tauri::RunEvent::ExitRequested { api, .. } => {
            api.prevent_exit();
        }
        _ => {}
    });
}

async fn notification_task(app: AppHandle) {
    loop {
        tokio::time::sleep(time::Duration::from_secs(60 * 60)).await;
        create_drink_notification(app.clone())
    }
}

async fn oauth_server_cb(app: AppHandle) {
    let http_server = tiny_http::Server::http("localhost:11132").unwrap();
    println!("HTTP Server now listening on {}", http_server.server_addr());

    for request in http_server.incoming_requests() {
        let req_url = request.url();
        println!("Received request: {}", req_url);

        if !req_url.starts_with("/?") {
            request
                .respond(tiny_http::Response::from_string("404 Not Found").with_status_code(404))
                .ok();

            continue;
        }

        let qs = querystring::querify(&req_url[2..]);
        let code = qs.iter().find(|(k, _v)| *k == "code");

        if code.is_none() {
            request
                .respond(tiny_http::Response::from_string("400 Bad Request").with_status_code(400))
                .ok();

            continue;
        }

        println!("Received code: {}", code.unwrap().1);
        app.get_window("oauth").and_then(|w| w.close().ok());

        request
            .respond(tiny_http::Response::from_string(
                "Loading... You may close this window",
            ))
            .ok();

        break;
    }

    println!("Shutting down HTTP Server");
    http_server.unblock();
    std::mem::drop(http_server);
}
