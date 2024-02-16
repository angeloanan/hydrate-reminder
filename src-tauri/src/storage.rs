use std::{
    fs::File,
    io::Write,
    sync::{LazyLock, RwLock},
};

use capnp::message::{ReaderOptions, TypedReader};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use tracing::trace;

use crate::{app_capnp::app_state, structs::drink_point::DrinkPoint};

#[cfg(debug_assertions)]
static PROJECT_DIR: LazyLock<ProjectDirs> =
    LazyLock::new(|| ProjectDirs::from("fyi", "angelo", "hydrate-reminder-dev").unwrap());
#[cfg(not(debug_assertions))]
static PROJECT_DIR: LazyLock<ProjectDirs> =
    LazyLock::new(|| ProjectDirs::from("fyi", "angelo", "hydrate-reminder").unwrap());

#[derive(Serialize, Deserialize, Debug)]
pub struct InnerAppState {
    pub version: u16,
    pub has_onboarded: bool,

    pub drink_history: Vec<DrinkPoint>,
}

pub struct AppState(pub RwLock<InnerAppState>);

const INITIAL_APP_STATE: InnerAppState = InnerAppState {
    version: 1,
    has_onboarded: false,

    drink_history: vec![],
};

fn parse_saved_data(bytes: &[u8]) -> InnerAppState {
    let saved_data = capnp::serialize_packed::read_message(bytes, ReaderOptions::default())
        .expect("Unable to serialize saved app data!");
    let saved_data_reader = TypedReader::<_, app_state::Owned>::new(saved_data);

    let saved_data_owned = saved_data_reader.get().unwrap();

    // Don't forget to check if struct exists or not using `has`
    InnerAppState {
        version: saved_data_owned.get_version(),
        has_onboarded: saved_data_owned.get_has_onboarded(),

        drink_history: saved_data_owned
            .get_drink_history()
            .unwrap()
            .iter()
            .map(|drink_point| DrinkPoint {
                timestamp: drink_point.get_timestamp(),
                amount: drink_point.get_amount(),
            })
            .collect(),
    }
}

fn serialize_app_state(state: &InnerAppState) -> Vec<u8> {
    let mut message = capnp::message::TypedBuilder::<app_state::Owned>::new_default();
    let mut app_state_builder = message.init_root();

    app_state_builder.set_version(state.version);
    app_state_builder.set_has_onboarded(state.has_onboarded);

    let mut drink_history_builder =
        app_state_builder.init_drink_history(
            u32::try_from(state.drink_history.len())
                .expect("Unable to convert drink history length to u32. Did you change this to string or do you have > 4 billion drink points?")
        );

    for (i, drink_point) in state.drink_history.iter().enumerate() {
        let mut drink_point_builder = drink_history_builder
            .reborrow()
            .get(u32::try_from(i).unwrap());
        drink_point_builder.set_timestamp(drink_point.timestamp);
        drink_point_builder.set_amount(drink_point.amount);
    }

    let mut serialized_data = Vec::new();
    capnp::serialize_packed::write_message(&mut serialized_data, message.borrow_inner())
        .expect("Unable to serialize app state!");

    serialized_data
}

pub fn get_saved_data() -> InnerAppState {
    let data_path = PROJECT_DIR.data_dir().join("history.bin");
    trace!("Data path: {data_path:?}");

    if !PROJECT_DIR
        .data_dir()
        .try_exists()
        .expect("Unable to check if data directory exists. Please elevate the app's permission!")
    {
        std::fs::create_dir_all(PROJECT_DIR.data_dir()).unwrap();
    };

    if data_path.exists() {
        let binary_data = std::fs::read(data_path).expect("Unable to read data file!");

        return parse_saved_data(&binary_data);
    }

    // If the data file doesn't exist, create it and write the initial data to it
    save_app_state(&INITIAL_APP_STATE).expect("Unable to write initial data to file!");
    INITIAL_APP_STATE
}

pub fn save_app_state(state: &InnerAppState) -> Result<usize, std::io::Error> {
    let data_path = PROJECT_DIR.data_dir().join("history.bin");

    let binary_data = serialize_app_state(state);
    File::create(data_path)
        .expect("Unable to create data file!")
        .write(&binary_data)
}
