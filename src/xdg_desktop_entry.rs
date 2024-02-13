
use std::rc::Rc;
use std::path::Path;
use gio::DesktopAppInfo;

#[derive(Debug)]
pub struct XdgDesktopEntry {
	display_name: String,
	keywords: Vec<String>,
	app_info: Rc<DesktopAppInfo>,
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
		let display_name =  match app_info.generic_name() {
			Some(name) => name.to_string(),
			None => app_info.filename()
				.expect("filename was passed to constructor").into_os_string().into_string()
				.expect("filename must not contain invalid character range")
		};
		
		Some(XdgDesktopEntry {
			display_name,
			app_info: app_info.into(),
			keywords
		})
	}	
}

