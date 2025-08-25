use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{Local, Utc};
use discord_rich_presence::activity::{self as drp_activity, ActivityType, Timestamps};

#[repr(u8)]
#[derive(Default, Clone)]
pub enum TimestampType {
    #[default]
    SinceStart = 1,
    LocalTime = 2,
    CustomTimestamp = 3,
    SinceLastUpdate = 4,
}

#[derive(Clone)]
pub struct Activity {
    pub id: Option<String>,
    pub activity_type: ActivityType,
    pub details: Option<String>,
    pub state: Option<String>,
    pub party_size: Option<u8>,
    pub party_max: Option<u8>,
    pub timestamp_type: TimestampType,
    pub large_key: Option<String>,
    pub small_key: Option<String>,
    pub small_text: Option<String>,
    pub large_text: Option<String>,
    pub button1_text: Option<String>,
    pub button2_text: Option<String>,
    pub button1_url: Option<String>,
    pub button2_url: Option<String>,
}

impl Default for Activity {
    fn default() -> Self {
        Self {
            id: Default::default(),
            activity_type: ActivityType::Playing,
            details: Default::default(),
            state: Default::default(),
            party_size: Default::default(),
            party_max: Default::default(),
            timestamp_type: Default::default(),
            large_key: Default::default(),
            small_key: Default::default(),
            small_text: Default::default(),
            large_text: Default::default(),
            button1_text: Default::default(),
            button2_text: Default::default(),
            button1_url: Default::default(),
            button2_url: Default::default(),
        }
    }
}
