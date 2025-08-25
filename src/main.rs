mod activity;
mod presence;

use dark_light::Mode;
use iced::daemon::DefaultStyle;
use iced::futures::SinkExt;
use iced::futures::channel::mpsc::{self, UnboundedSender};
use iced::widget::{button, center};
use iced::{Element, Task, Theme};

use crate::activity::Activity;
use crate::presence::{Presence, PresenceThreadMessage};
struct App {
	pub send: UnboundedSender<MainThreadMessage>,
	pub activity: Activity,
}
#[derive(Debug, Clone)]
enum Message {
	PresenceMessage(PresenceThreadMessage),
	Test,
	Error(String),
}

impl From<PresenceThreadMessage> for Message {
	fn from(val: PresenceThreadMessage) -> Self {
		Message::PresenceMessage(val)
	}
}

#[allow(clippy::large_enum_variant)]
pub enum MainThreadMessage {
	Start,
	Stop,
	SetActivity(Activity),
	Close,
}

#[tokio::main]
async fn main() -> iced::Result {
	let (main_send, presence_recv) = mpsc::unbounded::<MainThreadMessage>();
	let (presence_send, main_recv) = mpsc::unbounded::<PresenceThreadMessage>();
	Presence::spawn_thread(presence_send, presence_recv);
	iced::application("Discord presence", App::update, App::view)
		.theme(|_| match dark_light::detect() {
			Ok(Mode::Light) => Theme::Light,
			_ => Theme::Dark,
		})
		.run_with(move || {
			(
				App {
					send: main_send,
					activity: Activity::default(),
				},
				Task::stream(main_recv).map(|v| v.into()),
			)
		})
}

impl App {
	fn update(&mut self, message: Message) -> Task<Message> {
		match message {
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

	fn view(&self) -> Element<'_, Message> {
		center(button("Test").on_press(Message::Test))
			.padding(20)
			.into()
	}
}
