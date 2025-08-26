use anyhow::anyhow;
use chrono::{Local, Timelike, Utc};
use discord_rich_presence::{self as drp, DiscordIpc, DiscordIpcClient, activity::Timestamps};
use iced::futures::{
	SinkExt, StreamExt,
	channel::mpsc::{UnboundedReceiver, UnboundedSender},
};
use log::error;
use thiserror::Error;
use tokio::task;

use crate::{MainThreadMessage, activity::TimestampType};

#[derive(Debug, Clone)]
pub enum PresenceThreadMessage {
	Err(String),
}
pub struct Presence {
	pub recv: UnboundedReceiver<MainThreadMessage>,
	pub send: UnboundedSender<PresenceThreadMessage>,
	pub client: Option<DiscordIpcClient>,
	pub close: bool,
}

#[derive(Error, Debug)]
enum PresenceError {
	#[error("No id provided")]
	NoIdError,
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
			MainThreadMessage::Start => {
				let Some(client) = &mut self.client else {
					return Err(anyhow!(PresenceError::NoIdError));
				};
				client.connect()?;
			}
			MainThreadMessage::Stop => {
				let Some(client) = &mut self.client else {
					return Err(anyhow!(PresenceError::NoIdError));
				};
				client.close()?;
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
						let now = Local::now();
						let offset_seconds: i64 = now.second() as i64
							+ (now.minute() as i64 * 60)
							+ (now.hour() as i64 * 3600);

						let mut t = Timestamps::new();
						t.start = Some(Utc::now().timestamp() - offset_seconds);

						t
					}
					TimestampType::LocalTime => {
						let now = Local::now();
						let offset_seconds: i64 = now.second() as i64
							+ (now.minute() as i64 * 60)
							+ (now.hour() as i64 * 3600);

						let mut t = Timestamps::new();
						t.start = Some(offset_seconds);

						t
					}
					TimestampType::CustomTimestamp => todo!(),
					TimestampType::SinceLastUpdate => todo!(),
				};
				let discord_activity = drp::activity::Activity {
					state: activity.state.as_deref(),
					details: activity.details.as_deref(),
					timestamps: Some(timestamp),
					..Default::default()
				};
				client.connect()?;
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
