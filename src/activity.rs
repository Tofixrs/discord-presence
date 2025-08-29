use chrono::{DateTime, Utc};
use discord_rich_presence::activity::ActivityType;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[repr(u8)]
#[derive(Default, Clone, Debug, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
pub enum TimestampType {
	#[default]
	SinceStart = 1,
	LocalTime = 2,
	Custom = 3,
	SinceLastUpdate = 4,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
#[serde(rename = "Preset")]
pub struct Activity {
	#[serde(rename = "ID")]
	pub id: Option<String>,
	#[serde(rename = "Type")]
	pub activity_type: ActivityType,
	pub details: Option<String>,
	pub state: Option<String>,
	pub party_size: Option<i32>,
	pub party_max: Option<i32>,
	#[serde(rename = "Timestamps")]
	pub timestamp_type: TimestampType,
	#[serde(with = "crp_format")]
	pub custom_timestamp: Option<DateTime<Utc>>,
	pub large_key: Option<String>,
	pub small_key: Option<String>,
	pub small_text: Option<String>,
	pub large_text: Option<String>,
	pub button1_text: Option<String>,
	pub button2_text: Option<String>,
	#[serde(rename = "Button1URL")]
	pub button1_url: Option<String>,
	#[serde(rename = "Button2URL")]
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
			custom_timestamp: Default::default(),
		}
	}
}

mod crp_format {
	use chrono::{DateTime, NaiveDateTime, Utc};
	use serde::{self, Deserialize, Deserializer, Serializer};

	const FORMAT: &str = "%Y-%m-%dT%H:%M:%S";

	// The signature of a serialize_with function must follow the pattern:
	//
	//    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
	//    where
	//        S: Serializer
	//
	// although it may also be generic over the input types T.
	pub fn serialize<S>(date: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		match date {
			Some(date) => {
				let s = format!("{}", date.format(FORMAT));
				serializer.serialize_str(&s)
			}
			None => serializer.serialize_none(),
		}
	}

	// The signature of a deserialize_with function must follow the pattern:
	//
	//    fn deserialize<'de, D>(D) -> Result<T, D::Error>
	//    where
	//        D: Deserializer<'de>
	//
	// although it may also be generic over the output types T.
	pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
	where
		D: Deserializer<'de>,
	{
		let s: Option<String> = Option::deserialize(deserializer)?;
		match s {
			Some(s) => {
				let dt =
					NaiveDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)?;
				Ok(Some(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc)))
			}
			None => Ok(None),
		}
	}
}
