use chrono::{Datelike, Local, Timelike};
use iced::widget::{column, row};
use iced::{
	Alignment, Element, Length,
	alignment::{Horizontal, Vertical},
	widget::{button, container, pick_list, radio, text, text_input},
	window::Id,
};
use iced_aw::{
	Menu, date_picker,
	helpers::time_picker,
	menu_bar, menu_items, number_input,
	time_picker::{Period, Time},
};

use crate::{
	ACTIVITY_TYPES, TEXT_COLUMN_WIDTH,
	activity::TimestampType,
	app::{
		App, ConnectionState,
		message::{ActivityMsg, Message},
	},
};

impl App {
	pub fn view(&self, _window: Id) -> Element<'_, Message> {
		let menu_tpl = |items| Menu::new(items).max_width(50.).offset(15.0);
		#[rustfmt::skip]
		let mb = menu_bar!(
        (menu_button("File"), {menu_tpl(menu_items!(
            (b("Save", Message::None))
            (b("Save", Message::Exit))
        ))})
    );

		column![
			mb,
			column![
				self.id_row(),
				self.details_row(),
				self.state_row(),
				self.timestamp_row(),
				self.connect_row(),
			]
			.padding(10.)
			.spacing(10.)
			.align_x(Horizontal::Center),
			row![]
		]
		.width(Length::Fill)
		.height(Length::Fill)
		.into()
	}
	fn id_row(&self) -> Element<'_, Message, iced::Theme, iced::Renderer> {
		row![
			text("ID")
				.align_x(Alignment::End)
				.width(Length::Fixed(TEXT_COLUMN_WIDTH)),
			text_input("", &self.activity.id.clone().unwrap_or_default())
				.on_input(|v| ActivityMsg::Id(v).into()),
			text("Type"),
			pick_list(
				ACTIVITY_TYPES.clone(),
				Some(self.activity.activity_type.clone()),
				|v| { ActivityMsg::ActivityType(v).into() }
			)
		]
		.spacing(10.)
		.align_y(Vertical::Center)
		.width(Length::Fill)
		.into()
	}
	fn details_row(&self) -> Element<'_, Message, iced::Theme, iced::Renderer> {
		row![
			text("Details")
				.align_x(Alignment::End)
				.width(Length::Fixed(TEXT_COLUMN_WIDTH)),
			text_input("", &self.activity.details.clone().unwrap_or_default())
				.on_input(|v| ActivityMsg::Detials(v).into()),
		]
		.spacing(10.)
		.align_y(Vertical::Center)
		.width(Length::Fill)
		.into()
	}
	fn state_row(&self) -> Element<'_, Message, iced::Theme, iced::Renderer> {
		row![
			text("State")
				.align_x(Alignment::End)
				.width(Length::Fixed(TEXT_COLUMN_WIDTH)),
			text_input("", self.activity.state.as_ref().unwrap_or(&String::new()))
				.on_input(|v| ActivityMsg::State(v).into()),
			text("Party"),
			number_input(
				&self.activity.party_size.unwrap_or_default(),
				0..=self.activity.party_max.unwrap_or_default(),
				|v| ActivityMsg::PartySize(v).into(),
			),
			text("of"),
			number_input(
				&self.activity.party_max.unwrap_or_default(),
				0..=i32::MAX,
				|v| ActivityMsg::PartyMax(v).into(),
			)
		]
		.spacing(10.)
		.align_y(Vertical::Center)
		.width(Length::Fill)
		.into()
	}
	fn timestamp_row(&self) -> Element<'_, Message, iced::Theme, iced::Renderer> {
		let open_date_picker = button("Choose date").on_press(Message::ChooseDate);
		let open_time_picker = button("Choose time").on_press(Message::ChooseTime);
		let now = Local::now();
		let custom_date = date_picker(
			self.show_date_picker,
			self.activity.custom_date.unwrap_or(date_picker::Date {
				year: now.year(),
				month: now.month(),
				day: now.day(),
			}),
			open_date_picker,
			Message::CancelDate,
			|v| ActivityMsg::CustomDate(v).into(),
		);
		let custom_time = time_picker(
			self.show_time_picker,
			self.activity.custom_time.unwrap_or(Time::Hms {
				hour: now.hour(),
				minute: now.minute(),
				second: now.second(),
				period: Period::H24,
			}),
			open_time_picker,
			Message::CancelTime,
			|v| ActivityMsg::CustomTime(v).into(),
		)
		.use_24h()
		.show_seconds();
		row![
			text("Timestamp")
				.align_x(Alignment::End)
				.align_y(Alignment::Center)
				.height(Length::Fill)
				.width(Length::Fixed(TEXT_COLUMN_WIDTH)),
			column![
				radio(
					"Since last presence update",
					TimestampType::SinceLastUpdate,
					Some(self.activity.timestamp_type),
					|v| ActivityMsg::TimestampType(v).into()
				),
				radio(
					"Since start",
					TimestampType::SinceStart,
					Some(self.activity.timestamp_type),
					|v| ActivityMsg::TimestampType(v).into()
				),
				radio(
					"Local time",
					TimestampType::LocalTime,
					Some(self.activity.timestamp_type),
					|v| ActivityMsg::TimestampType(v).into()
				),
				row![
					radio(
						"Custom",
						TimestampType::Custom,
						Some(self.activity.timestamp_type),
						|v| ActivityMsg::TimestampType(v).into()
					),
					custom_date,
					custom_time
				]
				.align_y(Vertical::Center)
				.spacing(10.),
			]
			.width(Length::Fill)
			.spacing(10.)
		]
		.height(Length::Shrink)
		.spacing(10.)
		.align_y(Vertical::Bottom)
		.into()
	}
	fn connect_row(&self) -> Element<'_, Message, iced::Theme, iced::Renderer> {
		let (text, msg) = match self.connection_state {
			ConnectionState::Disconnected => ("Connect", Message::Connect),
			ConnectionState::Connecting => ("Connecting", Message::None),
			ConnectionState::Connected => ("Disconnect", Message::Disconnect),
		};
		container(
			row![
				button(text).on_press(msg),
				button("Set activity").on_press(Message::SetActivity)
			]
			.spacing(10.),
		)
		.height(Length::Fill)
		.align_y(Vertical::Bottom)
		.align_x(Horizontal::Center)
		.into()
	}
}

fn b(label: &str, msg: Message) -> button::Button<'_, Message, iced::Theme, iced::Renderer> {
	button(label).on_press(msg).width(Length::Fill)
}
fn menu_button(label: &str) -> button::Button<'_, Message, iced::Theme, iced::Renderer> {
	button(label).on_press(Message::None)
}

