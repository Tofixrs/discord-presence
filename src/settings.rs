use std::fs;

use anyhow::{Context, anyhow};
use image::EncodableLayout;
use serde::{Deserialize, Serialize};

use crate::{activity::Activity, app::App};

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Settings {}

#[derive(Serialize, Deserialize, Default)]
pub struct SettingsFile {
	pub settings: Settings,
	pub activity: Activity,
}

static APP_ID: &str = "me.tofixrs.discord-presence";

impl SettingsFile {
	fn get_location() -> std::path::PathBuf {
		path::storage_dir(APP_ID)
			.expect("Unsupported platform")
			.join("settings.xml")
	}
	pub async fn write(&self) -> anyhow::Result<()> {
		let text = serde_xml_rs::to_string(self)?;
		let folder = path::storage_dir(APP_ID).ok_or(anyhow!("Unsupported platform"))?;
		tokio::fs::create_dir_all(folder).await?;
		tokio::fs::write(Self::get_location(), text).await?;

		Ok(())
	}

	pub fn read() -> anyhow::Result<SettingsFile> {
		if !Self::get_location().exists() {
			return Ok(SettingsFile::default());
		};

		let data = fs::read(Self::get_location())?;

		serde_xml_rs::from_reader::<SettingsFile, _>(data.as_bytes())
			.context("Failed to parse settings file")
	}
}

impl From<&App> for SettingsFile {
	fn from(value: &App) -> Self {
		SettingsFile {
			settings: value.settings.clone(),
			activity: value.activity.clone(),
		}
	}
}

mod path {
	/*
	* Copyright (c) 2018-2021 Emil Ernerfeldt <emil.ernerfeldt@gmail.com>

	Permission is hereby granted, free of charge, to any
	person obtaining a copy of this software and associated
	documentation files (the "Software"), to deal in the
	Software without restriction, including without
	limitation the rights to use, copy, modify, merge,
	publish, distribute, sublicense, and/or sell copies of
	the Software, and to permit persons to whom the Software
	is furnished to do so, subject to the following
	conditions:

	The above copyright notice and this permission notice
	shall be included in all copies or substantial portions
	of the Software.

	THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
	ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
	TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
	PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
	SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
	CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
	OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
	IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
	DEALINGS IN THE SOFTWARE.
	  */
	use std::{
		collections::HashMap,
		env::home_dir,
		io::Write as _,
		path::{Path, PathBuf},
	};
	enum OperatingSystem {
		Nix,
		Windows,
		Mac,
		Unknown,
	}
	impl OperatingSystem {
		pub const fn from_target_os() -> Self {
			if cfg!(target_os = "macos") {
				Self::Mac
			} else if cfg!(target_os = "windows") {
				Self::Windows
			} else if cfg!(target_os = "linux")
				|| cfg!(target_os = "dragonfly")
				|| cfg!(target_os = "freebsd")
				|| cfg!(target_os = "netbsd")
				|| cfg!(target_os = "openbsd")
			{
				Self::Nix
			} else {
				Self::Unknown
			}
		}
	}

	/// On native, the path is:
	/// * Linux:   `/home/UserName/.config/APP_ID`
	/// * macOS:   `/Users/UserName/Library/Preferences/APP_ID`
	/// * Windows: `C:\Users\UserName\AppData\Roaming\APP_ID\data`
	pub fn storage_dir(app_id: &str) -> Option<PathBuf> {
		use std::env::var_os;
		match OperatingSystem::from_target_os() {
			OperatingSystem::Nix => var_os("XDG_CONFIG_HOME")
				.map(PathBuf::from)
				.filter(|p| p.is_absolute())
				.or_else(|| home_dir().map(|p| p.join(".config")))
				.map(|p| {
					p.join(
						app_id
							.to_lowercase()
							.replace(|c: char| c.is_ascii_whitespace(), ""),
					)
				}),
			OperatingSystem::Mac => home_dir().map(|p| {
				p.join("Library")
					.join("Preferences")
					.join(app_id.replace(|c: char| c.is_ascii_whitespace(), "-"))
			}),
			OperatingSystem::Windows => roaming_appdata().map(|p| p.join(app_id).join("data")),
			OperatingSystem::Unknown => None,
		}
	}

	// Adapted from
	// https://github.com/rust-lang/cargo/blob/6e11c77384989726bb4f412a0e23b59c27222c34/crates/home/src/windows.rs#L19-L37
	#[cfg(all(windows, not(target_vendor = "uwp")))]
	#[expect(unsafe_code)]
	fn roaming_appdata() -> Option<PathBuf> {
		use std::ffi::OsString;
		use std::os::windows::ffi::OsStringExt as _;
		use std::ptr;
		use std::slice;

		use windows_sys::Win32::Foundation::S_OK;
		use windows_sys::Win32::System::Com::CoTaskMemFree;
		use windows_sys::Win32::UI::Shell::{
			FOLDERID_RoamingAppData, KF_FLAG_DONT_VERIFY, SHGetKnownFolderPath,
		};

		unsafe extern "C" {
			fn wcslen(buf: *const u16) -> usize;
		}
		let mut path_raw = ptr::null_mut();

		// SAFETY: SHGetKnownFolderPath allocates for us, we don't pass any pointers to it.
		// See https://learn.microsoft.com/en-us/windows/win32/api/shlobj_core/nf-shlobj_core-shgetknownfolderpath
		let result = unsafe {
			SHGetKnownFolderPath(
				&FOLDERID_RoamingAppData,
				KF_FLAG_DONT_VERIFY as u32,
				std::ptr::null_mut(),
				&mut path_raw,
			)
		};

		let path = if result == S_OK {
			// SAFETY: SHGetKnownFolderPath indicated success and is supposed to allocate a null-terminated string for us.
			let path_slice = unsafe { slice::from_raw_parts(path_raw, wcslen(path_raw)) };
			Some(PathBuf::from(OsString::from_wide(path_slice)))
		} else {
			None
		};

		// SAFETY:
		// This memory got allocated by SHGetKnownFolderPath, we didn't touch anything in the process.
		// A null ptr is a no-op for `CoTaskMemFree`, so in case this failed we're still good.
		// https://learn.microsoft.com/en-us/windows/win32/api/combaseapi/nf-combaseapi-cotaskmemfree
		unsafe { CoTaskMemFree(path_raw.cast()) };

		path
	}

	#[cfg(any(not(windows), target_vendor = "uwp"))]
	fn roaming_appdata() -> Option<PathBuf> {
		None
	}
}
