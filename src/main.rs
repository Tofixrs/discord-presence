mod activity;
mod presence;
mod tray;

use dark_light::Mode;
use iced::futures::SinkExt;
use iced::futures::channel::mpsc::{self, UnboundedReceiver, UnboundedSender};
use iced::widget::{button, center};
use iced::window::{Id, Settings};
use iced::{Element, Subscription, Task, Theme, window};
use tray_icon::{MouseButton, MouseButtonState};

use crate::activity::Activity;
use crate::presence::{Presence, PresenceThreadMessage};
use crate::tray::{Tray, TrayMessage};
struct App {
	send: UnboundedSender<MainThreadMessage>,
	activity: Activity,
	window_visible: bool,
}
#[derive(Debug, Clone)]
enum Message {
	PresenceMessage(PresenceThreadMessage),
	TrayMessage(TrayMessage),
	Test,
	Error(String),
}

impl From<PresenceThreadMessage> for Message {
	fn from(val: PresenceThreadMessage) -> Self {
		Message::PresenceMessage(val)
	}
}

impl From<TrayMessage> for Message {
	fn from(val: TrayMessage) -> Self {
		Message::TrayMessage(val)
	}
}

#[allow(clippy::large_enum_variant)]
pub enum MainThreadMessage {
	Start,
	Stop,
	SetActivity(Activity),
	Exit,
}

#[tokio::main]
async fn main() -> iced::Result {
	tracing_subscriber::fmt::init();
	iced::daemon(App::new, App::update, App::view)
		.title("Discord presence")
		.theme(|_, _| match dark_light::detect() {
			Ok(Mode::Light) => Theme::Light,
			_ => Theme::Dark,
		})
		.run()
}

impl App {
	fn new() -> (Self, Task<Message>) {
		let (main_send, presence_recv) = mpsc::unbounded::<MainThreadMessage>();
		let (presence_send, main_recv) = mpsc::unbounded::<PresenceThreadMessage>();
		let (tray_send, tray_recv) = mpsc::unbounded::<TrayMessage>();
		Presence::spawn_thread(presence_send, presence_recv);
		Tray::spawn_thread(tray_send);
		let (_id, open) = window::open(Settings::default());

		(
			App {
				send: main_send,
				activity: Activity::default(),
				window_visible: false,
			},
			Task::batch([
				Task::stream(main_recv).map(|v| v.into()),
				Task::stream(tray_recv).map(|v| v.into()),
				open.then(|_| Task::none()),
			]),
		)
	}
	fn update(&mut self, message: Message) -> Task<Message> {
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
					let (_id, open) = window::open(Settings::default());

					open.then(|_| Task::none())
				}
			}
			Message::TrayMessage(TrayMessage::Open) => {
				let (_id, open) = window::open(Settings::default());

				open.then(|_| Task::none())
			}
			Message::TrayMessage(TrayMessage::Exit) => {
				let mut sender = self.send.clone();
				Task::batch([
					iced::exit(),
					Task::future(async move { sender.send(MainThreadMessage::Exit).await })
						.then(|_| Task::none()),
				])
			}
			Message::Test => {
				self.activity.id = Some(String::from("1009390231100866580"));
				self.activity.state = Some(String::from("test"));

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
			_ => Task::none(),
		}
	}

	fn view(&self, _window: Id) -> Element<'_, Message> {
		center(button("Test").on_press(Message::Test))
			.padding(20)
			.into()
	}
}
