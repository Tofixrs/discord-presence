pub mod message;
pub mod update;
pub mod view;

use std::process;

use iced::{
	Task,
	futures::{
		SinkExt,
		channel::mpsc::{self, UnboundedSender},
	},
	window,
};

use crate::{
	activity::Activity,
	app::message::{MainThreadMessage, Message},
	presence::{Presence, PresenceThreadMessage},
	settings::{Settings, SettingsFile},
	tray::{Tray, TrayMessage},
};

pub struct App {
	send: UnboundedSender<MainThreadMessage>,
	pub activity: Activity,
	window_visible: bool,
	connection_state: ConnectionState,
	show_date_picker: bool,
	show_time_picker: bool,
	pub settings: Settings,
}

pub enum ConnectionState {
	Disconnected,
	Connecting,
	Connected,
}

impl App {
	pub fn new() -> (Self, Task<Message>) {
		let (main_send, presence_recv) = mpsc::unbounded::<MainThreadMessage>();
		let (presence_send, main_recv) = mpsc::unbounded::<PresenceThreadMessage>();
		let (tray_send, tray_recv) = mpsc::unbounded::<TrayMessage>();
		Presence::spawn_thread(presence_send, presence_recv);
		Tray::spawn_thread(tray_send);

		match SettingsFile::read() {
			Ok(data) => (
				App {
					send: main_send,
					activity: data.activity,
					window_visible: false,
					connection_state: ConnectionState::Disconnected,
					show_date_picker: false,
					show_time_picker: false,
					settings: data.settings,
				},
				Task::batch([
					Task::stream(main_recv).map(|v| v.into()),
					Task::stream(tray_recv).map(|v| v.into()),
					Self::open_window(),
				]),
			),
			Err(err) => {
				let err_string = err.to_string();

				rfd::MessageDialog::new()
					.set_title("Failed to open settings file")
					.set_description(format!("Error\n\n{err_string}"))
					.set_level(rfd::MessageLevel::Error)
					.show();

				process::exit(1);
			}
		}
	}
	fn open_window() -> Task<Message> {
		let (_id, open) = window::open(window::Settings {
			..Default::default()
		});

		open.then(|_| Task::none())
	}
	fn send_presence_msg(&mut self, msg: MainThreadMessage) -> Task<Message> {
		let mut sender = self.send.clone();
		Task::future(async move { sender.send(msg).await }).then(|v| {
			let Err(err) = v else {
				return Task::none();
			};

			Task::done(Message::Error(err.to_string()))
		})
	}
	fn write_settings(&self) -> Task<Message> {
		let file = SettingsFile::from(self);

		Task::future(async move { file.write().await }).then(|v| {
			let Err(err) = v else {
				return Task::none();
			};

			Task::done(Message::Error(err.to_string()))
		})
	}
}
