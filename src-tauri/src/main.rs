#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]
#![warn(clippy::perf)]
#![warn(clippy::complexity)]
#![warn(clippy::style)]
#![feature(lazy_cell)]
#![allow(clippy::redundant_pub_crate)]
// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod http;
mod oauth;
mod sound;
mod storage;
mod structs;

use {structs::drink_point::DrinkPoint, tauri::Position};

use std::sync::RwLock;

use commands::create_drink_notification;
use rodio::{OutputStream, Sink};
use sound::drink_audio;
use storage::AppState;
use tauri::{
    AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, WindowBuilder,
};
use tokio::{select, time};

#[cfg(debug_assertions)]
const PROJECT_IDENTIFIER: &str = "fyi.angelo.hydrate-reminder-dev";
#[cfg(not(debug_assertions))]
const PROJECT_IDENTIFIER: &str = "fyi.angelo.hydrate-reminder";

// Required by Cap'n Proto
pub mod app_capnp {
    include!(concat!(env!("OUT_DIR"), "/schema/app_capnp.rs"));
}

fn spawn_main_window(app: &AppHandle) {
    if let Some(main_window) = app.get_window("main") {
        return main_window
            .set_focus()
            .expect("Unable to focus main window!");
    }

    let window = WindowBuilder::new(app, "main", tauri::WindowUrl::App("index.html".into()))
        .title("Hydrate Reminder")
        .inner_size(300.0, 500.0)
        .position(1_000_000.0, 1_000_000.0)
        .resizable(false)
        .closable(true)
        .always_on_top(true)
        .build()
        .expect("Unable to create a new window!");

    let monitor = window.current_monitor().unwrap().unwrap();
    let w = monitor.size().width - (300.0 * monitor.scale_factor()) as u32;

    window
        .set_position(Position::Physical({
            tauri::PhysicalPosition {
                x: i32::try_from(w).unwrap(),
                y: 0,
            }
        }))
        .expect("Unable to set window position!");

    // Close the window when it loses focus ON PROD
    #[cfg(not(debug_assertions))]
    {
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
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
fn submit_drink(app: &AppHandle, amount: f64) {
    let state = app.state::<AppState>();

    // Add a new drink point to the history & drop the lock
    {
        let mut app_state = state.0.write().unwrap();
        app_state.drink_history.push(DrinkPoint::new(amount));
    }

    storage::save_app_state(&state.0.read().unwrap()).unwrap();

    app.emit_all("drink", ()).unwrap();
    app.trigger_global("drink", None);

    play_drink_sound();
}

fn play_drink_sound() {
    tauri::async_runtime::spawn(async move {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        sink.append(drink_audio());
        sink.sleep_until_end();
    });
}

fn handle_tray_event(app: &AppHandle, event: SystemTrayEvent) {
    match event {
        tauri::SystemTrayEvent::LeftClick { position, .. } => {}
        tauri::SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
            "drink-full" => submit_drink(app, 200.0),
            "drink-half" => submit_drink(app, 100.0),
            "open-settings" => spawn_main_window(app),

            "quit" => app.exit(0),

            _ => {
                println!("Unknown menu item clicked: {id}");
            }
        },
        _ => (),
    }
}

fn main() {
    // Setup notifications on macos
    #[cfg(target_os = "macos")]
    {
        use mac_notification_sys::get_bundle_identifier_or_default;

        mac_notification_sys::set_application(
            get_bundle_identifier_or_default("hydrate-reminder").as_str(),
        )
        .unwrap();
    }

    let tray_menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("drink-full", "🥛 Drink (200ml)"))
        .add_item(CustomMenuItem::new("drink-half", "💧 Sip (100ml)"))
        .add_native_item(tauri::SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("open-settings", "Settings"))
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

    println!("Loaded app state: {app_state:#?}");

    let mut app = tauri::Builder::default()
        .manage(AppState(RwLock::new(app_state)))
        .system_tray(tray)
        .on_system_tray_event(handle_tray_event)
        .invoke_handler(tauri::generate_handler![
            commands::create_drink_notification,
            commands::list_drinks,
            commands::list_drinks_group_day,
            commands::get_latest_drink,
            oauth::start_oauth_authentication
        ])
        .build(tauri::generate_context!())
        .expect("Error while running tauri application");

    #[cfg(target_os = "macos")]
    {
        app.set_activation_policy(tauri::ActivationPolicy::Accessory);
    }

    tauri::async_runtime::spawn(notification_task_manager(app.app_handle()));

    app.run(|_, e| {
        if let tauri::RunEvent::ExitRequested { api, .. } = e {
            api.prevent_exit();
        }
    });
}

async fn notification_task_manager(app: AppHandle) {
    // A channel to short-circuit the notification task
    let (sender, mut receiver) = tokio::sync::mpsc::channel::<()>(1);

    app.listen_global("drink", move |_e| {
        tauri::async_runtime::block_on(async { sender.send(()).await.unwrap() });
    });

    loop {
        println!("[Re-]scheduling notification task");

        select! {
            () = schedule_notification_task(app.clone()) => {
                println!("Notification task completed, rescheduling");
            },
            _ = receiver.recv() => {
                println!("Received drink event, rescheduling notification task");
            },
        };
    }
}

async fn schedule_notification_task(app: AppHandle) {
    tokio::time::sleep(time::Duration::from_secs(60 * 60)).await;

    create_drink_notification();
}
