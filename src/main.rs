mod activity;
mod presence;
mod tray;

use dark_light::Mode;
use discord_rich_presence::activity::ActivityType;
use iced::alignment::{Horizontal, Vertical};
use iced::futures::SinkExt;
use iced::futures::channel::mpsc::{self, UnboundedSender};
use iced::widget::{button, column, container, pick_list, row, text, text_input};
use iced::window::{Id, Settings};
use iced::{Element, Font, Length, Task, Theme, window};
use iced_aw::{Menu, SelectionList, menu_bar, menu_items, selection_list, style};
use log::error;
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
	ActivityMsg(PresetMsg),
	None,
	Connect,
	SetActivity,
}

#[derive(Debug, Clone)]
enum PresetMsg {
	Id(String),
	ActivityType(ActivityType),
}

impl From<PresenceThreadMessage> for Message {
	fn from(val: PresenceThreadMessage) -> Self {
		Message::PresenceMessage(val)
	}
}
impl From<PresetMsg> for Message {
	fn from(val: PresetMsg) -> Self {
		Message::ActivityMsg(val)
	}
}

impl From<TrayMessage> for Message {
	fn from(val: TrayMessage) -> Self {
		Message::TrayMessage(val)
	}
}

#[allow(clippy::large_enum_variant)]
pub enum MainThreadMessage {
	Connect(String),
	Disconnect,
	SetActivity(Activity),
	Exit,
}

static ACTIVITY_TYPES: [ActivityType; 4] = [
	ActivityType::Playing,
	ActivityType::Watching,
	ActivityType::Competing,
	ActivityType::Listening,
];

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
			Message::PresenceMessage(PresenceThreadMessage::Err(err)) => {
				error!("{err}");
				Task::none()
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
				let mut sender = self.send.clone();
				let mut sender2 = self.send.clone();
				let activity = self.activity.clone();
				let Some(id) = &activity.id else {
					return Task::done(Message::Error(String::from("No id")));
				};
				let id = id.clone();
				Task::future(async move { sender2.send(MainThreadMessage::Connect(id)).await })
					.then(|v| {
						let Err(err) = v else {
							return Task::none();
						};

						Task::done(Message::Error(err.to_string()))
					})
					.chain(
						Task::future(async move {
							sender.send(MainThreadMessage::SetActivity(activity)).await
						})
						.then(|v| {
							let Err(err) = v else {
								return Task::none();
							};

							Task::done(Message::Error(err.to_string()))
						}),
					)
			}
			Message::ActivityMsg(msg) => {
				match msg {
					PresetMsg::Id(v) => {
						let _ = self.activity.id.insert(v);
					}
					PresetMsg::ActivityType(activity_type) => {
						self.activity.activity_type = activity_type;
					}
				};

				Task::none()
			}
			_ => Task::none(),
		}
	}

	fn view(&self, _window: Id) -> Element<'_, Message> {
		let menu_tpl = |items| Menu::new(items).max_width(50.).offset(15.0);
		#[rustfmt::skip]
		let mb = menu_bar!(
        (menu_button("File"), {menu_tpl(menu_items!(
            (b("Save", Message::Test))
        ))})
    );

		column![
			mb,
			column![
				row![
					text("Id"),
					text_input("", &self.activity.id.clone().unwrap_or_default())
						.on_input(|v| PresetMsg::Id(v).into()),
					pick_list(
						ACTIVITY_TYPES.clone(),
						Some(self.activity.activity_type.clone()),
						|v| { PresetMsg::ActivityType(v).into() }
					)
				]
				.spacing(10.)
				.align_y(Vertical::Center),
				container(
					row![
						button("Connect").on_press(Message::Connect),
						button("Set activity").on_press(Message::SetActivity)
					]
					.spacing(10.)
				)
				.height(Length::Fill)
				.align_y(Vertical::Bottom)
				.align_x(Horizontal::Center)
			]
			.padding(10.)
			.align_x(Horizontal::Center),
			row![]
		]
		.width(Length::Fill)
		.height(Length::Fill)
		.into()
	}
}
fn b(label: &str, msg: Message) -> button::Button<'_, Message, iced::Theme, iced::Renderer> {
	button(label).on_press(msg).width(Length::Fill)
}
fn menu_button(label: &str) -> button::Button<'_, Message, iced::Theme, iced::Renderer> {
	button(label).on_press(Message::None)
}
