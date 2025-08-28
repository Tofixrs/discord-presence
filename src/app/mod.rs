pub mod message;
pub mod update;
pub mod view;

use iced::{
	Task,
	futures::{
		SinkExt,
		channel::mpsc::{self, UnboundedSender},
	},
	window::{self, Settings},
};

use crate::{
	activity::Activity,
	app::message::{MainThreadMessage, Message},
	presence::{Presence, PresenceThreadMessage},
	tray::{Tray, TrayMessage},
};

pub struct App {
	send: UnboundedSender<MainThreadMessage>,
	activity: Activity,
	window_visible: bool,
	connection_state: ConnectionState,
	show_date_picker: bool,
	show_time_picker: bool,
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

		(
			App {
				send: main_send,
				activity: Activity::default(),
				window_visible: false,
				connection_state: ConnectionState::Disconnected,
				show_date_picker: false,
				show_time_picker: false,
			},
			Task::batch([
				Task::stream(main_recv).map(|v| v.into()),
				Task::stream(tray_recv).map(|v| v.into()),
				Self::open_window(),
			]),
		)
	}
	fn open_window() -> Task<Message> {
		let (_id, open) = window::open(Settings {
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
}
