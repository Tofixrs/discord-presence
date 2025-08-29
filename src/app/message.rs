use discord_rich_presence::activity::ActivityType;

use crate::{
	activity::{Activity, TimestampType},
	presence::PresenceThreadMessage,
	tray::TrayMessage,
};

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum Message {
	Presence(PresenceThreadMessage),
	TrayMessage(TrayMessage),
	Error(String),
	Activity(ActivityMsg),
	None,
	Connect,
	SetActivity,
	Disconnect,
	ChooseDate,
	ChooseTime,
	CancelTime,
	CancelDate,
	Exit,
	OpenActivity,
	SaveActivity,
	LoadActivity(Activity),
}

#[allow(clippy::large_enum_variant)]
pub enum MainThreadMessage {
	Connect(String),
	Disconnect,
	SetActivity(Activity),
	Exit,
}

#[derive(Debug, Clone)]
pub enum ActivityMsg {
	Id(String),
	Detials(String),
	State(String),
	TimestampType(TimestampType),
	ActivityType(ActivityType),
	PartySize(i32),
	PartyMax(i32),
	CustomDate(iced_aw::date_picker::Date),
	CustomTime(iced_aw::time_picker::Time),
	Button1Text(String),
	Button2Text(String),
	Button1URL(String),
	Button2URL(String),
	SmallImageText(String),
	SmallImageKey(String),
	LargeImageText(String),
	LargeImageKey(String),
}

impl From<PresenceThreadMessage> for Message {
	fn from(val: PresenceThreadMessage) -> Self {
		Message::Presence(val)
	}
}
impl From<ActivityMsg> for Message {
	fn from(val: ActivityMsg) -> Self {
		Message::Activity(val)
	}
}

impl From<TrayMessage> for Message {
	fn from(val: TrayMessage) -> Self {
		Message::TrayMessage(val)
	}
}
