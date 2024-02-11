use std::collections::HashMap;

use chrono::DateTime;
use rodio::{OutputStream, Sink};
use tauri::{AppHandle, Manager};

use crate::{sound::notification_audio, storage::AppState, structs::drink_point::DrinkPoint};

#[tauri::command]
pub fn create_drink_notification() {
    tauri::async_runtime::spawn(async move {
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

        #[cfg(target_os = "windows")]
        {
            winrt_notification::Toast::new(&app.config().tauri.bundle.identifier)
                .title("Time to drink!")
                .text1("It's been 1 hour since your last drink, time to drink again!")
                .duration(winrt_notification::Duration::Short)
                .sound(None)
                .show()
                .unwrap();
        }

        // TODO: Add Linux support

        sink.sleep_until_end();
    });
}

#[tauri::command]
pub fn get_latest_drink(app: AppHandle) -> Option<DrinkPoint> {
    println!("[get_latest_drink] Sending latest drink data to FEnd");

    let state = app.state::<AppState>();
    let app_state = state.0.read().unwrap();

    app_state.drink_history.last().copied()
}

#[tauri::command]
pub fn list_drinks(state: tauri::State<AppState>) -> Vec<DrinkPoint> {
    println!("[list_drinks] Sending drink data to FEnd");

    state.0.read().unwrap().drink_history.clone()
}

#[tauri::command]
pub fn list_drinks_group_day(state: tauri::State<AppState>) -> HashMap<String, f64> {
    println!("[list_drinks_group_day] Sending drink data to FEnd");

    let drink_history = state.0.read().unwrap().drink_history.clone();

    let mut grouped_drinks: HashMap<String, f64> = HashMap::new();

    // Iterate through drinks, group drinks by day - DrinkPoint timestamp is set to 00:00:00
    for point in &drink_history {
        let local_datetime = DateTime::from_timestamp(point.timestamp, 0)
            .unwrap()
            .naive_local()
            .date();

        let entry = grouped_drinks
            .entry(local_datetime.to_string())
            .or_insert(0.0);
        *entry += point.amount;
    }

    grouped_drinks
}
