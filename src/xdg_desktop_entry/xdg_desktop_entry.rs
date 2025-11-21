use std::rc::Rc;
use std::path::Path;

use gtk::prelude::{AppInfoExt, AppLaunchContextExt};
use gio::{AppInfo, AppLaunchContext, DesktopAppInfo};

#[derive(Debug)]
pub struct XdgDesktopEntry {
	pub display_name: String,
	pub keywords: Vec<String>,
	pub path: Box<Path>,
	pub app_info: Rc<DesktopAppInfo>,
}

impl XdgDesktopEntry {
	pub fn try_from(path: &Path) -> Option<Self> {			
		let app_info = if let Some(app_info) = DesktopAppInfo::from_filename(path) {
			app_info
		}  else {
			return None;
		};

		let keywords: Vec<String> = app_info.keywords()
				.iter().map(|g_string| g_string.to_string()).collect();
		if let Some(bool_setting) = app_info.locale_string("NoDisplay") {
			if bool_setting == "true" {
				return None;
			}
		}

		let display_name =  match app_info.locale_string("Name") {
			Some(name) => name.to_string(),
			None => app_info.filename()
				.expect("filename was passed to constructor").into_os_string().into_string()
				.expect("filename must not contain invalid character range")
		};
		
		Some(XdgDesktopEntry {
			display_name,
			path: path.into(),
			app_info: app_info.into(),
			keywords
		})
	}	

	pub fn on_app_launch(
			_: &AppLaunchContext, 
			app_info: &AppInfo, 
			launched_event: &gtk::glib::Variant	) {
		println!("Application launched: {:?} {:?}", launched_event, app_info);
	}

	pub fn launch(&self, action: Option<&str>) {
		let launch_context = gio::AppLaunchContext::new();
		launch_context.connect_launched(Self::on_app_launch);
		println!("{:?}",self.path);
		match action {
			Some(action) => self.app_info.launch_action(action, Some(&launch_context)),
			None => self.app_info.launch(&[], Some(&launch_context)).expect("REASON")
		};
	}
}
