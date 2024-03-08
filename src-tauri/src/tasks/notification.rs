use std::sync::Arc;

use chrono::{Days, Duration, Local};
use tauri::{AppHandle, Manager};
use tokio::select;
use tracing::{instrument, trace};

use crate::{
    commands::create_drink_notification, storage::AppState, structs::drink_point::DrinkPoint,
};

#[instrument(skip(app))]
pub async fn task_manager(app: AppHandle) {
    // A channel to short-circuit the notification task
    let notify = Arc::new(tokio::sync::Notify::new());

    let notifier = notify.clone();
    app.listen_global("drink", move |_e| {
        trace!("Received drink event. Sending reschedule signal");
        notifier.notify_one();
    });

    let notified = notify.clone();
    loop {
        trace!("Re-scheduling notification task");
        // Debounce 1s
        tokio::time::sleep(Duration::seconds(1).to_std().unwrap()).await;

        select! {
            () = wait_next_notif(app.clone()) => {
                create_drink_notification(app.clone());
                trace!("Notification task completed, rescheduling");
            },
            _ = notified.notified() => {
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

    trace!("Seconds until next drink: {time_difference} seconds");

    if time_difference.num_seconds() < 0 {
        // If the time difference is negative, we've already passed the next drink time
        // so we'll just wait indefinitely
        //
        // TODO: Handle this edge case in the future, maybe set an hourly / daily reminder

        let start_day_tomorrow = chrono::Local::now()
            .naive_local()
            .checked_add_days(Days::new(1))
            .unwrap()
            .date()
            .and_hms_opt(10, 0, 0)
            .unwrap()
            .and_local_timezone(chrono::Local)
            .unwrap();
        trace!("10AM Tomorrow: {start_day_tomorrow}");
        let time_until_start_day_tomorrow = start_day_tomorrow - Local::now();
        trace!("Seconds until 10AM tomorrow: {time_until_start_day_tomorrow}");

        tokio::time::sleep(time_until_start_day_tomorrow.to_std().unwrap()).await;
    } else {
        tokio::time::sleep(time_difference.to_std().unwrap()).await;
    }
}
