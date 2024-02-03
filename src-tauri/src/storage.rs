use std::{
    fs::File,
    io::Write,
    sync::{LazyLock, RwLock},
};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::structs::drink_point::DrinkPoint;

static PROJECT_DIR: LazyLock<ProjectDirs> =
    LazyLock::new(|| ProjectDirs::from("fyi", "angelo", "hydrate-reminder").unwrap());

#[derive(Serialize, Deserialize, Debug)]
pub struct InnerAppState {
    pub has_onboarded: bool,

    pub drink_history: Vec<DrinkPoint>,
}

pub struct AppState(pub RwLock<InnerAppState>);

const INITIAL_APP_STATE: InnerAppState = InnerAppState {
    has_onboarded: false,
    drink_history: vec![],
};

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

        return bincode::deserialize(&binary_data).expect("Unable to deserialize data. Did you change the data file manually or did the data get corrupted?");
    }

    // If the data file doesn't exist, create it and write the initial data to it
    let initial_data_serialized = bincode::serialize(&INITIAL_APP_STATE).unwrap();

    File::create(data_path)
        .expect("Unable to create data file!")
        .write(&initial_data_serialized)
        .expect("Unable to write initial data to file!");

    INITIAL_APP_STATE
}

pub fn save_app_state(state: &InnerAppState) -> Result<(), std::io::Error> {
    let data_path = PROJECT_DIR.data_dir().join("history.bin");
    let binary_data = bincode::serialize(state).expect("Unable to serialize data!");

    std::fs::write(data_path, binary_data)
}
