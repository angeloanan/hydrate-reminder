use {
    chrono::Utc,
    serde::{Deserialize, Serialize},
};

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
            amount: 200.0,
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
