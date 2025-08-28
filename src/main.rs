mod activity;
mod app;
mod presence;
mod tray;

use dark_light::Mode;
use discord_rich_presence::activity::ActivityType;
use iced::Theme;
use iced_aw::ICED_AW_FONT_BYTES;

use crate::app::App;

const TEXT_COLUMN_WIDTH: f32 = 100.;

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
		.font(ICED_AW_FONT_BYTES)
		.run()
}
