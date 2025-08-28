use discord_rich_presence::activity::ActivityType;

#[repr(u8)]
#[derive(Default, Clone, Debug, Copy, PartialEq, Eq)]
pub enum TimestampType {
	#[default]
	SinceStart = 1,
	LocalTime = 2,
	Custom = 3,
	SinceLastUpdate = 4,
}

#[derive(Clone)]
pub struct Activity {
	pub id: Option<String>,
	pub activity_type: ActivityType,
	pub details: Option<String>,
	pub state: Option<String>,
	pub party_size: Option<i32>,
	pub party_max: Option<i32>,
	pub timestamp_type: TimestampType,
	pub custom_date: Option<iced_aw::date_picker::Date>,
	pub custom_time: Option<iced_aw::time_picker::Time>,
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
			custom_date: Default::default(),
			custom_time: Default::default(),
		}
	}
}
