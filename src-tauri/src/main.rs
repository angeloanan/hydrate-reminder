#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]
#![warn(clippy::perf)]
#![warn(clippy::complexity)]
#![warn(clippy::style)]
#![feature(lazy_cell)]
// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod oauth;
mod sound;
mod storage;
mod structs;

use {
    chrono::{prelude::*, Duration},
    mac_notification_sys::get_bundle_identifier_or_default,
    std::ops::Add,
    structs::drink_point::DrinkPoint,
    tauri::Position,
};

use {std::sync::RwLock, tauri::State};

use rodio::{OutputStream, Sink};
use sound::{drink_audio, notification_audio};
use storage::AppState;
use tauri::{
    AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, WindowBuilder,
};
use tokio::time;

#[cfg(debug_assertions)]
const PROJECT_IDENTIFIER: &str = "fyi.angelo.hydrate-reminder-dev";
#[cfg(not(debug_assertions))]
const PROJECT_IDENTIFIER: &str = "fyi.angelo.hydrate-reminder";

// Required by Cap'n Proto
pub mod app_capnp {
    include!(concat!(env!("OUT_DIR"), "/schema/app_capnp.rs"));
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {name}! You've been greeted from Rust!")
}

fn spawn_main_window(app: &AppHandle) {
    if let Some(main_window) = app.get_window("main") {
        main_window
            .set_focus()
            .expect("Unable to focus main window!");
    } else {
        let window = WindowBuilder::new(app, "main", tauri::WindowUrl::App("index.html".into()))
            .hidden_title(true)
            .inner_size(300.0, 500.0)
            .resizable(false)
            .closable(true)
            .build()
            .expect("Unable to create a new window!");

        let monitor = window.current_monitor().unwrap().unwrap();
        let w = monitor.size().width - 300;
        window
            .set_position(Position::Physical({
                tauri::PhysicalPosition {
                    x: i32::try_from(w).unwrap(),
                    y: 0,
                }
            }))
            .expect("Unable to set window position!");

        let app_handle = app.clone();
        window.on_window_event(move |e| {
            if matches!(e, tauri::WindowEvent::Focused(false)) {
                println!("Closing window");
                app_handle
                    .get_window("main")
                    .unwrap()
                    .close()
                    .expect("Failed to close window!");
            }
        });
    }

    //     println!("Window event: {:?}", e);
    // });
}

#[tauri::command]
fn create_drink_notification() {
    tokio::spawn(async move {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        sink.append(notification_audio());

        #[cfg(target_os = "macos")]
        {
            mac_notification_sys::Notification::new()
                .app_icon("")
                .title("Time to drink!")
                .message("It's been 1 hour since your last drink, time to drink again!")
                .send()
                .unwrap();
        }

        sink.sleep_until_end();
    });
}

fn submit_drink(state: tauri::State<AppState>) {
    // Add a new drink point to the history & drop the lock
    {
        let mut app_state = state.0.write().unwrap();
        app_state.drink_history.push(DrinkPoint::new(200.0));
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
        sink.sleep_until_end();
    });
}

fn handle_tray_event(app: &AppHandle, event: SystemTrayEvent) {
    if let tauri::SystemTrayEvent::MenuItemClick { id, .. } = event {
        match id.as_str() {
            "drink" => submit_drink(app.state()),
            "open-settings" => spawn_main_window(app),

            "quit" => app.exit(0),

            _ => {
                println!("Unknown menu item clicked: {id}");
            }
        }
    }
}

#[tokio::main]
async fn main() {
    // Setup notifications on macos
    #[cfg(target_os = "macos")]
    {
        mac_notification_sys::set_application(
            get_bundle_identifier_or_default("hydrate-reminder").as_str(),
        )
        .unwrap();
    }

    let tray_menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("drink", "🥛 Drink"))
        .add_item(CustomMenuItem::new("open-settings", "Open Settings"))
        .add_native_item(tauri::SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("quit", "Quit"));

    let mut tray = SystemTray::new()
        .with_menu(tray_menu)
        .with_icon(tauri::Icon::Raw(
            include_bytes!("../icons/tray.png").to_vec(),
        ));

    #[cfg(target_os = "macos")]
    {
        tray = tray.with_icon_as_template(true);
    }

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

    app.run(|_, e| {
        if let tauri::RunEvent::ExitRequested { api, .. } = e {
            api.prevent_exit();
        }
    });
}

async fn notification_task(app: AppHandle) {
    let state: State<AppState> = app.state();

    // Recheck every 60 seconds
    loop {
        tokio::time::sleep(time::Duration::from_secs(60)).await;

        let app_state = state.0.read().unwrap();

        if let Some(point) = app_state.drink_history.last() {
            let last_drink = DateTime::from_timestamp(point.timestamp, 0).expect(
                "Invalid timestamp in drink history. Did you modify the data file manually?",
            );
            let now = chrono::Utc::now();
            let diff = now - last_drink;

            if diff > chrono::Duration::hours(1)
                && diff < chrono::Duration::hours(1).add(Duration::minutes(1))
            {
                create_drink_notification();
            }
        }
    }
}
