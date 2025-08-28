use std::path::Path;

use iced::futures::{SinkExt, channel::mpsc::UnboundedSender};
use log::error;
use tokio::task;
use tray_icon::{
	TrayIconBuilder, TrayIconEvent,
	menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
};

#[derive(Debug, Clone)]
pub enum TrayMessage {
	Err(String),
	TrayIcon(TrayIconEvent),
	Exit,
	Open,
}

pub struct Tray {
	send: UnboundedSender<TrayMessage>,
	close: bool,
}

impl Tray {
	pub fn spawn_thread(send: UnboundedSender<TrayMessage>) {
		std::thread::spawn(move || {
			#[cfg(target_os = "linux")]
			gtk::init().expect("Failed to init gtk");

			let menu = Menu::new();
			let open = MenuItem::with_id("open", "Open", true, None);
			let exit = MenuItem::with_id("exit", "Exit", true, None);
			menu.append_items(&[
				&PredefinedMenuItem::about(
					None,
					Some(AboutMetadata {
						name: Some("Discord presence".to_string()),
						..Default::default()
					}),
				),
				&PredefinedMenuItem::separator(),
				&open,
				&exit,
			])
			.expect("Failed to create tray menu");

			let _tray_icon = TrayIconBuilder::new()
				.with_title("Discord presence")
				.with_icon(load_icon(Path::new("./icon.png")))
				.with_menu(Box::new(menu))
				.build()
				.expect("Failed to create tray");

			#[cfg(target_os = "linux")]
			gtk::main();
		});
		task::spawn(async {
			let mut state = Tray { send, close: false };
			loop {
				let Err(err) = state.event_loop().await else {
					if state.close {
						return;
					};
					continue;
				};

				let Err(err) = state.send.send(TrayMessage::Err(err.to_string())).await else {
					continue;
				};

				error!("{err}");
			}
		});
	}

	async fn event_loop(&mut self) -> anyhow::Result<()> {
		if let Ok(event) = TrayIconEvent::receiver().try_recv() {
			self.send.send(TrayMessage::TrayIcon(event)).await?;
		}
		if let Ok(event) = MenuEvent::receiver().try_recv() {
			match event.id.as_ref() {
				"open" => {
					self.send.send(TrayMessage::Open).await?;
				}
				"exit" => {
					self.send.send(TrayMessage::Exit).await?;
					self.close = true;
				}
				_ => {}
			};
		}
		Ok(())
	}
}

fn load_icon(path: &std::path::Path) -> tray_icon::Icon {
	let (icon_rgba, icon_width, icon_height) = {
		let image = image::open(path)
			.expect("Failed to open icon path")
			.into_rgba8();
		let (width, height) = image.dimensions();
		let rgba = image.into_raw();
		(rgba, width, height)
	};
	tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}
