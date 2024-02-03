#![feature(lazy_cell)]
// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod oauth;
mod sound;
mod storage;

use std::sync::RwLock;

use rodio::{OutputStream, Sink};
use sound::{drink_audio, notification_audio};
use storage::AppState;
use tauri::{
    api::notification::Notification, AppHandle, CustomMenuItem, Manager, SystemTray,
    SystemTrayEvent, SystemTrayMenu, WindowBuilder,
};
use tokio::time;

use crate::storage::DrinkPoint;

const PROJECT_IDENTIFIER: &'static str = "fyi.angelo.hydrate-reminder";

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
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        sink.append(notification_audio());
        Notification::new(&app.config().tauri.bundle.identifier)
            .title("Time to drink!")
            .body("It's been 1 hour since your last drink, time to drink again!")
            .show()
            .expect("Unable to create drink notification!");

        sink.sleep_until_end();
    });
}

fn submit_drink(state: tauri::State<AppState>) {
    {
        // Add a new drink point to the history & drop the lock
        let mut app_state = state.0.write().unwrap();
        app_state.drink_history.push(DrinkPoint::default());
    }

    storage::save_app_state(&state.0.read().unwrap()).unwrap();

    play_drink_sound();
}

#[tauri::command]
fn list_drinks(state: tauri::State<AppState>) -> Vec<DrinkPoint> {
    println!("Sending drink data to fend");

    state.0.read().unwrap().drink_history.clone()
}

fn play_drink_sound() {
    tokio::spawn(async move {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        sink.append(drink_audio());
        sink.sleep_until_end()
    });
}

fn handle_tray_event(app: &AppHandle, event: SystemTrayEvent) {
    match event {
        tauri::SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
            "drink" => submit_drink(app.state()),
            "open-settings" => spawn_main_window(app),

            "quit" => app.exit(0),

            _ => {
                println!("Unknown menu item clicked: {}", id);
            }
        },
        _ => {}
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

    let app_state = storage::get_saved_data();

    let mut app = tauri::Builder::default()
        .manage(AppState(RwLock::new(app_state)))
        .system_tray(tray)
        .on_system_tray_event(handle_tray_event)
        .invoke_handler(tauri::generate_handler![
            create_drink_notification,
            list_drinks,
            greet,
            oauth::start_oauth_authentication
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
