
use gio::AppInfo;
use gtk::prelude::AppInfoExt;
use std::rc::Rc;
use std::path::Path;
use gio::DesktopAppInfo;
use gio::prelude::AppLaunchContextExt;
use gio::AppLaunchContext;

#[derive(Debug)]
pub struct XdgDesktopEntry {
	pub display_name: String,
	pub keywords: Vec<String>,
	pub path: Box<Path>,
	pub app_info: Rc<DesktopAppInfo>,
}

enum Property {
	Name,
	Exec,
	MimeType,
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
			launch_context: &AppLaunchContext, 
			app_info: &AppInfo, 
			launched_event: &gtk::glib::Variant	) {
		// https://docs.gtk.org/gio/signal.AppLaunchContext.launched.html get pid  
		println!("{:?}", launched_event);
	}

	pub fn launch(&self, action: Option<&str>) {
		let launch_context = gio::AppLaunchContext::new();
		launch_context.connect_launched(Self::on_app_launch);
		println!("{:?}",self.path);
		match action {
			Some(action) => self.app_info.launch_action(action, Some(&launch_context)),
			None => self.app_info.launch(&[], Some(&launch_context)).expect("REASON")

		}
	}
}

