// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fmt::format;

use directories::ProjectDirs;
use lazy_static::lazy_static;
use tauri::{
    api::notification::{Notification, Sound},
    App, AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, Window,
    WindowBuilder,
};

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn create_new_window(app: &AppHandle) -> Window {
    let window = WindowBuilder::new(app, "main", tauri::WindowUrl::App("index.html".into()))
        .hidden_title(true)
        .resizable(false)
        .decorations(false)
        .closable(true)
        .build()
        .expect("Unable to create a new window!");

    // window.on_window_event(|e| {
    //     match e {
    //         tauri::WindowEvent::Focused(false) => {
    //             window.close().expect("Failed to close window!");
    //         }
    //         _ => {}
    //     }

    //     println!("Window event: {:?}", e);
    // });

    window
}

#[tauri::command]
fn create_drink_notification(app: AppHandle) {
    println!("Creating drink notification");
    let notif_sound = Sound::Default;

    Notification::new(&app.config().tauri.bundle.identifier)
        .title("Time to drink!")
        .body("It's been 1 hour since your last drink, time to drink again!")
        .sound(notif_sound)
        .show()
        .expect("Unable to create drink notification!");
}

fn handle_tray_event(app: &AppHandle, event: SystemTrayEvent) {
    match event {
        tauri::SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
            "drink" => {
                println!("I have {} windows", app.windows().len());
                create_new_window(app);
            }

            "quit" => app.exit(0),

            _ => {
                println!("Unknown menu item clicked: {}", id);
            }
        },
        _ => {}
    }
}

async fn handle_oauth_flow(app: &AppHandle) {
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
        ("response_type", "token"),
        ("redirect_uri", "localhost:11132"),
        ("access_type", "online"),
    ]);
    let url = Url::parse(format!("https://accounts.google.com/o/oauth2/v2/auth?{}", qs).as_str())
        .unwrap();

    let window = WindowBuilder::new(app, "oauth", tauri::WindowUrl::External(url))
        .hidden_title(true)
        .resizable(false)
        .decorations(false)
        .closable(true)
        .build()
        .expect("Unable to create a new window!");
}

lazy_static! {
    static ref PROJECT_DIR: ProjectDirs =
        ProjectDirs::from("fyi", "angelo", "hydrate-reminder").unwrap();
}

#[tokio::main]
async fn main() {
    let tray_menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("drink", "Drink"))
        .add_native_item(tauri::SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("quit", "Quit"));

    let tray = SystemTray::new().with_menu(tray_menu);

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .system_tray(tray)
        .on_system_tray_event(handle_tray_event)
        .invoke_handler(tauri::generate_handler![create_drink_notification, greet])
        .build(tauri::generate_context!())
        .expect("Error while running tauri application")
        .run(|_, e| match e {
            tauri::RunEvent::ExitRequested { api, .. } => {
                api.prevent_exit();
            }
            _ => {}
        });
}
