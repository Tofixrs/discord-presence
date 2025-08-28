use anyhow::anyhow;
use chrono::{Local, TimeZone, Timelike, Utc};
use discord_rich_presence::{
	self as drp, DiscordIpc, DiscordIpcClient,
	activity::{Party, Timestamps},
};
use iced::futures::{
	SinkExt, StreamExt,
	channel::mpsc::{UnboundedReceiver, UnboundedSender},
};
use iced_aw::time_picker::{Period, Time};
use log::error;
use thiserror::Error;
use tokio::task;

use crate::activity::TimestampType;
use crate::app::message::MainThreadMessage;

#[derive(Debug, Clone)]
pub enum PresenceThreadMessage {
	Err(String),
	Connected,
	Disconnected,
}
pub struct Presence {
	pub recv: UnboundedReceiver<MainThreadMessage>,
	pub send: UnboundedSender<PresenceThreadMessage>,
	pub client: Option<DiscordIpcClient>,
	pub close: bool,
	pub start_time: i64,
}

#[derive(Error, Debug)]
enum PresenceError {
	#[error("No id provided")]
	NoIdError,
	#[error("Not connected")]
	NotConnected,
	#[error("No date")]
	NoDate,
}

impl Presence {
	pub fn spawn_thread(
		send: UnboundedSender<PresenceThreadMessage>,
		recv: UnboundedReceiver<MainThreadMessage>,
	) {
		task::spawn(async {
			let mut state = Presence {
				send,
				recv,
				client: None,
				close: false,
				start_time: Utc::now().timestamp(),
			};

			loop {
				let Err(err) = state.event_loop().await else {
					if state.close {
						return;
					};
					continue;
				};

				let Err(err) = state
					.send
					.send(PresenceThreadMessage::Err(err.to_string()))
					.await
				else {
					continue;
				};

				error!("{err}");
			}
		});
	}
	async fn event_loop(&mut self) -> anyhow::Result<()> {
		let Some(msg) = self.recv.next().await else {
			return Ok(());
		};

		match msg {
			MainThreadMessage::Connect(id) => {
				let mut client = self
					.client
					.take_if(|v| v.client_id == id)
					.unwrap_or(DiscordIpcClient::new(&id));

				client.connect()?;
				let _ = self.client.insert(client);

				self.send.send(PresenceThreadMessage::Connected).await?;
			}
			MainThreadMessage::Disconnect => {
				let Some(client) = &mut self.client else {
					//this shouldnt fail but just in case ig, better then a .expect cuz it wont panic
					return Err(anyhow!(PresenceError::NotConnected));
				};
				client.close()?;
				self.send.send(PresenceThreadMessage::Disconnected).await?;
			}
			MainThreadMessage::SetActivity(activity) => {
				let Some(id) = activity.id else {
					return Err(anyhow!(PresenceError::NoIdError));
				};

				let mut client = self
					.client
					.take_if(|v| v.client_id == id)
					.unwrap_or(DiscordIpcClient::new(&id));

				let timestamp = match activity.timestamp_type {
					TimestampType::SinceStart => {
						let mut t = Timestamps::new();
						t.start = Some(self.start_time);

						t
					}
					TimestampType::LocalTime => {
						let now = Local::now();
						let offset_seconds: i64 = now.second() as i64
							+ (now.minute() as i64 * 60)
							+ (now.hour() as i64 * 3600);

						let mut t = Timestamps::new();
						t.start = Some(Utc::now().timestamp() - offset_seconds);

						t
					}
					TimestampType::Custom => {
						let (
							&Some(date),
							&Some(Time::Hms {
								hour,
								minute,
								second,
								period: Period::H24,
							}),
						) = (&activity.custom_date, &activity.custom_time)
						else {
							return Err(anyhow!(PresenceError::NoDate));
						};
						let time = Utc
							.with_ymd_and_hms(date.year, date.month, date.day, hour, minute, second)
							.unwrap();

						let mut t = Timestamps::new();
						t.start = Some(time.timestamp());
						t
					}
					TimestampType::SinceLastUpdate => {
						let mut t = Timestamps::new();
						t.start = Some(Utc::now().timestamp());
						t
					}
				};
				let discord_activity = drp::activity::Activity {
					state: activity.state.as_deref(),
					details: activity.details.as_deref(),
					timestamps: Some(timestamp),
					activity_type: Some(activity.activity_type),
					party: match (activity.party_max, activity.party_size) {
						(Some(max), Some(size)) => Some(Party {
							id: None,
							size: Some([size, max]),
						}),
						_ => None,
					},
					..Default::default()
				};
				client.set_activity(discord_activity)?;
				self.client = Some(client);
			}
			MainThreadMessage::Exit => {
				self.close = true;
			}
		}
		Ok(())
	}
}
