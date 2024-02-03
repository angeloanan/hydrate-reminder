use std::{
    fs::File,
    io::Write,
    sync::{LazyLock, RwLock},
};

use chrono::prelude::*;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

static PROJECT_DIR: LazyLock<ProjectDirs> =
    LazyLock::new(|| ProjectDirs::from("fyi", "angelo", "hydrate-reminder").unwrap());

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct DrinkPoint {
    /// Timestamp of when the drink was recorded
    pub timestamp: i64,

    /// Amount of water drank in milliliters
    pub amount: f64,
}

impl Default for DrinkPoint {
    fn default() -> Self {
        Self {
            timestamp: Utc::now().timestamp(),
            amount: 500.0,
        }
    }
}

impl DrinkPoint {
    pub fn new(amount: f64) -> Self {
        Self {
            timestamp: Utc::now().timestamp(),
            amount,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InnerAppState {
    pub drink_history: Vec<DrinkPoint>,
}

pub struct AppState(pub RwLock<InnerAppState>);

pub fn get_saved_data() -> InnerAppState {
    let data_path = PROJECT_DIR.data_dir().join("history.bin");
    println!("Data path: {:?}", data_path);

    if PROJECT_DIR
        .data_dir()
        .try_exists()
        .expect("Unable to check if data directory exists. Please elevate the app's permission!")
        == false
    {
        std::fs::create_dir_all(PROJECT_DIR.data_dir()).unwrap();
    };

    if data_path.exists() {
        let binary_data = std::fs::read(data_path).expect("Unable to read data file!");

        bincode::deserialize(&binary_data).expect("Unable to deserialize data. Did you change the data file manually or did the data get corrupted?")
    } else {
        let initial_data = InnerAppState {
            drink_history: vec![],
        };
        let initial_data_serialized = bincode::serialize(&initial_data).unwrap();

        File::create(data_path)
            .expect("Unable to create data file!")
            .write(&initial_data_serialized)
            .expect("Unable to write initial data to file!");

        initial_data
    }
}

pub fn save_app_state(state: &InnerAppState) -> Result<(), std::io::Error> {
    let data_path = PROJECT_DIR.data_dir().join("history.bin");
    let binary_data = bincode::serialize(state).expect("Unable to serialize data!");

    std::fs::write(data_path, binary_data)
}
