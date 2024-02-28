use chrono::{Duration, Local, Timelike};
use tauri::{AppHandle, Manager};
use tokio::select;
use tracing::{instrument, trace, warn};

use crate::{
    commands::create_drink_notification, storage::AppState, structs::drink_point::DrinkPoint,
};

#[instrument(skip(app))]
pub async fn task_manager(app: AppHandle) {
    // A channel to short-circuit the notification task
    let (sender, mut receiver) = tokio::sync::mpsc::channel::<()>(1);

    app.listen_global("drink", move |_e| {
        trace!("Received drink event. Sending reschedule signal");
        tauri::async_runtime::block_on(async {
            match sender.try_send(()) {
                Ok(_) => trace!("Rescheduled drink notification"),
                Err(e) => warn!("Failed to reschedule drink notification: {e}"),
            }
        });
    });

    loop {
        trace!("Re-scheduling notification task");
        // Debounce 1s
        tokio::time::sleep(Duration::seconds(1).to_std().unwrap()).await;

        select! {
            () = wait_next_notif(app.clone()) => {
                create_drink_notification(app.clone());
                trace!("Notification task completed, rescheduling");
            },
            _ = receiver.recv() => {
                trace!("Received drink event, rescheduling notification task");
            },
        };
    }
}

#[instrument(skip(app))]
async fn wait_next_notif(app: AppHandle) {
    let last_drink_timestamp = {
        let state = app.state::<AppState>();
        let app_state = state.0.read().unwrap();
        app_state
            .drink_history
            .last()
            .unwrap_or(&DrinkPoint::default())
            .timestamp
    };
    let last_drink_time = chrono::DateTime::from_timestamp(last_drink_timestamp, 0).unwrap();
    let next_drink_time = last_drink_time + chrono::Duration::hours(1);
    let time_difference = next_drink_time - chrono::Utc::now();

    if time_difference.num_seconds() < 0 {
        // If the time difference is negative, we've already passed the next drink time
        // so we'll just wait indefinitely
        //
        // TODO: Handle this edge case in the future, maybe set an hourly / daily reminder

        let mut start_day_tomorrow = chrono::Local::now() + Duration::days(1);
        start_day_tomorrow = start_day_tomorrow.with_hour(10).unwrap();
        let time_until_start_day_tomorrow = start_day_tomorrow - Local::now();

        tokio::time::sleep(time_until_start_day_tomorrow.to_std().unwrap()).await;
    } else {
        tokio::time::sleep(time_difference.to_std().unwrap()).await;
    }
}
