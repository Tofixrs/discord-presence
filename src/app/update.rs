use chrono::{Datelike, Timelike, Utc};
use iced::{Task, futures::SinkExt, window};
use iced_aw::time_picker::Time;
use log::{error, info};
use tray_icon::{MouseButton, MouseButtonState};

use crate::{
	activity::Activity,
	app::{
		App, ConnectionState,
		message::{ActivityMsg, MainThreadMessage, Message},
	},
	presence::PresenceThreadMessage,
	tray::TrayMessage,
};

impl App {
	pub fn update(&mut self, message: Message) -> Task<Message> {
		match message {
			Message::TrayMessage(TrayMessage::TrayIcon(tray_icon::TrayIconEvent::Click {
				button: MouseButton::Left,
				button_state: MouseButtonState::Down,
				..
			})) => {
				self.window_visible = !self.window_visible;

				if self.window_visible {
					window::get_latest().and_then(window::close)
				} else {
					Self::open_window()
				}
			}
			Message::TrayMessage(TrayMessage::Err(err)) => {
				error!("{err}");
				Task::none()
			}
			Message::TrayMessage(TrayMessage::Open) => Self::open_window(),
			Message::TrayMessage(TrayMessage::Exit) => {
				let mut sender = self.send.clone();
				Task::batch([
					iced::exit(),
					Task::future(async move { sender.send(MainThreadMessage::Exit).await })
						.then(|_| Task::none()),
				])
			}
			Message::TrayMessage(_) => Task::none(),
			Message::Presence(PresenceThreadMessage::Err(err)) => {
				if matches!(self.connection_state, ConnectionState::Connecting) {
					self.connection_state = ConnectionState::Disconnected;
				}
				error!("{err}");
				Task::none()
			}
			Message::Presence(PresenceThreadMessage::Connected) => {
				self.connection_state = ConnectionState::Connected;

				Task::none()
			}
			Message::Presence(PresenceThreadMessage::Disconnected) => {
				self.connection_state = ConnectionState::Disconnected;

				Task::none()
			}
			Message::Exit => Task::none(),
			Message::Error(err) => {
				error!("{err}");
				Task::none()
			}
			Message::SetActivity => {
				let mut sender = self.send.clone();
				let activity = self.activity.clone();

				Task::future(
					async move { sender.send(MainThreadMessage::SetActivity(activity)).await },
				)
				.then(|v| {
					let Err(err) = v else {
						return Task::none();
					};

					Task::done(Message::Error(err.to_string()))
				})
			}
			Message::Connect => {
				let activity = self.activity.clone();
				let Some(id) = &activity.id else {
					return Task::done(Message::Error(String::from("No id")));
				};
				let id = id.clone();
				self.connection_state = ConnectionState::Connecting;
				self.send_presence_msg(MainThreadMessage::Connect(id))
					.chain(self.send_presence_msg(MainThreadMessage::SetActivity(activity)))
			}
			Message::Disconnect => self.send_presence_msg(MainThreadMessage::Disconnect),
			Message::Activity(msg) => {
				match msg {
					ActivityMsg::Id(v) => {
						let _ = self.activity.id.insert(v);
					}
					ActivityMsg::ActivityType(activity_type) => {
						self.activity.activity_type = activity_type;
					}
					ActivityMsg::Detials(details) => {
						if details.is_empty() {
							self.activity.details = None
						}
						let _ = self.activity.details.insert(details);
					}
					ActivityMsg::State(state) => {
						if state.is_empty() {
							self.activity.state = None
						}
						let _ = self.activity.state.insert(state);
					}
					ActivityMsg::TimestampType(timestamp_type) => {
						self.activity.timestamp_type = timestamp_type;
					}
					ActivityMsg::PartySize(size) => {
						if size == 0 {
							self.activity.party_size = None;
						}
						let _ = self.activity.party_size.insert(size);
					}
					ActivityMsg::PartyMax(size) => {
						if size == 0 {
							self.activity.party_max = None;
						}
						let _ = self.activity.party_max.insert(size);
					}
					ActivityMsg::CustomDate(date) => {
						let timestamp = self.activity.custom_timestamp.unwrap_or(Utc::now());
						let timestamp = timestamp.with_year(date.year).unwrap();
						let timestamp = timestamp.with_month(date.month).unwrap();
						let timestamp = timestamp.with_day(date.day).unwrap();
						let _ = self.activity.custom_timestamp.insert(timestamp);
						self.show_date_picker = false;
					}
					ActivityMsg::CustomTime(time) => {
						let timestamp = self.activity.custom_timestamp.unwrap_or(Utc::now());
						let Time::Hms {
							hour,
							minute,
							second,
							period: _,
						} = time
						else {
							return Task::none();
						};
						let timestamp = timestamp.with_hour(hour).unwrap();
						let timestamp = timestamp.with_minute(minute).unwrap();
						let timestamp = timestamp.with_second(second).unwrap();

						let _ = self.activity.custom_timestamp.insert(timestamp);
						self.show_time_picker = false;
					}
					ActivityMsg::Button1Text(v) => {
						let _ = self.activity.button1_text.insert(v);
					}
					ActivityMsg::Button2Text(v) => {
						let _ = self.activity.button2_text.insert(v);
					}
					ActivityMsg::Button1URL(v) => {
						let _ = self.activity.button1_url.insert(v);
					}
					ActivityMsg::Button2URL(v) => {
						let _ = self.activity.button2_url.insert(v);
					}
					ActivityMsg::SmallImageText(v) => {
						let _ = self.activity.small_text.insert(v);
					}
					ActivityMsg::SmallImageKey(v) => {
						let _ = self.activity.small_key.insert(v);
					}
					ActivityMsg::LargeImageText(v) => {
						let _ = self.activity.large_text.insert(v);
					}
					ActivityMsg::LargeImageKey(v) => {
						let _ = self.activity.large_key.insert(v);
					}
				};

				Task::none()
			}
			Message::ChooseDate => {
				self.show_date_picker = !self.show_date_picker;

				Task::none()
			}
			Message::ChooseTime => {
				self.show_time_picker = !self.show_time_picker;

				Task::none()
			}
			Message::CancelDate => {
				self.show_date_picker = false;
				Task::none()
			}
			Message::CancelTime => {
				self.show_time_picker = false;
				Task::none()
			}
			Message::None => Task::none(),
			Message::OpenActivity => Task::future(async {
				let fd = rfd::AsyncFileDialog::new()
					.add_filter("activity (.crp)", &["crp"])
					.pick_file()
					.await;

				let Some(fd) = fd else {
					return Message::None;
				};

				let res = tokio::fs::read(fd.path()).await;
				match res {
					Ok(data) => {
						let activity = serde_xml_rs::from_reader::<Activity, _>(data.as_slice());
						match activity {
							Ok(v) => {
								info!("{v:?}");
								Message::LoadActivity(v)
							}
							Err(err) => Message::Error(err.to_string()),
						}
					}
					Err(err) => Message::Error(err.to_string()),
				}
			}),
			Message::SaveActivity => match serde_xml_rs::to_string(&self.activity) {
				Ok(activity) => Task::future(async move {
					let fd = rfd::AsyncFileDialog::new()
						.add_filter("activity (.crp)", &["crp"])
						.set_file_name("preset.crp")
						.save_file()
						.await;

					let Some(fd) = fd else {
						return Message::None;
					};
					let res = fd.write(activity.as_bytes()).await;
					let Err(err) = res else {
						return Message::None;
					};

					Message::Error(err.to_string())
				}),
				Err(err) => Task::done(Message::Error(err.to_string())),
			},
			Message::LoadActivity(activity) => {
				self.activity = activity;

				Task::none()
			}
		}
	}
}
