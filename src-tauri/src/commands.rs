use std::collections::HashMap;

use chrono::DateTime;
use rodio::{cpal::traits::HostTrait, OutputStream, Sink};
use tauri::{AppHandle, Manager};
use tracing::{error, instrument, trace, warn};

use crate::{sound::notification_audio, storage::AppState, structs::drink_point::DrinkPoint};

#[instrument(skip(app))]
#[tauri::command]
pub fn create_drink_notification(app: AppHandle) {
    if rodio::cpal::default_host()
        .output_devices()
        .unwrap()
        .count()
        > 1
    {
        tauri::async_runtime::spawn(async move {
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            let sink = Sink::try_new(&stream_handle).unwrap();

            sink.append(notification_audio());
            sink.sleep_until_end();
        });
    }

    #[cfg(target_os = "macos")]
    {
        if let Err(e) = mac_notification_sys::Notification::new()
            .app_icon("")
            .title("Time to drink!")
            .message("It's been 1 hour since your last drink, time to drink again!")
            .send()
        {
            error!("Failed to send drink notification: {e}");
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Err(e) = winrt_notification::Toast::new(&app.config().tauri.bundle.identifier)
            .title("Time to drink!")
            .text1("It's been 1 hour since your last drink, time to drink again!")
            .duration(winrt_notification::Duration::Short)
            .sound(None)
            .show()
        {
            error!("Failed to send drink notification: {e}");
        }
    }

    // TODO: Add Linux support
}

#[instrument(skip(app))]
#[tauri::command]
pub fn can_send_notification(app: tauri::AppHandle) -> bool {
    #[cfg(target_os = "windows")]
    {
        let is_focus_supported = windows::UI::Shell::FocusSessionManager::IsSupported().ok();
        let notification_state =
            unsafe { windows::Win32::UI::Shell::SHQueryUserNotificationState() }
                .expect("Failed to get Windows' notification state");
        let notif_status = notification_state.0;

        trace!("Focus session supported: {is_focus_supported:?}");
        trace!("Notification state: {notif_status}");

        match notif_status {
            // A screen saver is displayed, the machine is locked, or a nonactive Fast User Switching session is in progress.
            1 => {
                return false;
            }
            // A full-screen application is running or Presentation Settings are applied.
            // Presentation Settings allow a user to put their machine into a state fit for an uninterrupted presentation, such as a set of PowerPoint slides, with a single click.
            2 => {
                return false;
            }
            // A full-screen (exclusive mode) Direct3D application is running.
            3 => {
                return false;
            }
            // The user has activated Windows presentation settings to block notifications and pop-up messages.
            4 => {
                return false;
            }
            // None of the other states are found, notifications can be freely sent
            5 => {
                return true;
            }
            // Introduced in Windows 7. The current user is in "quiet time", which is the first hour after a new user logs into his or her account for the first time.
            // During this time, most notifications should not be sent or shown. This lets a user become accustomed to a new computer system without those distractions.
            // Quiet time also occurs for each user after an operating system upgrade or clean installation.
            //
            // Applications should set the NIIF_RESPECT_QUIET_TIME flag in their notifications or balloon tooltip,
            // which prevents those items from being displayed while the current user is in the quiet-time state.
            6 => {
                return false;
            }
            // A Windows Store app is running.
            7 => {
                return false;
            }
            _ => {
                warn!("Unknown notification state: {notif_status}");

                return false;
            }
        }
    }

    true
}

#[instrument(skip(app))]
#[tauri::command]
pub fn get_latest_drink(app: AppHandle) -> Option<DrinkPoint> {
    trace!("Sending latest drink data to FEnd");

    let state = app.state::<AppState>();
    let app_state = state.0.read().unwrap();

    app_state.drink_history.last().copied()
}

#[instrument(skip(state))]
#[tauri::command]
pub fn list_drinks(state: tauri::State<AppState>) -> Vec<DrinkPoint> {
    trace!("Sending drink data to FEnd");

    state.0.read().unwrap().drink_history.clone()
}

#[instrument(skip(state))]
#[tauri::command]
pub fn list_drinks_group_day(state: tauri::State<AppState>) -> HashMap<String, f64> {
    trace!("Sending drink data to FEnd");

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
