use crate::{Rc, Vec};
use crate::xdg_desktop_entry::XdgDesktopEntry;
use crate::search::SearchResult;

#[derive(Default)]
pub struct SearchContext {
	pub user_desktop_files: Rc<Vec<XdgDesktopEntry>>,
	pub(in super) result_cache: SearchResult,
}


impl SearchContext {
	pub fn set_desktop_files(&mut self, user_desktop_files: Rc<Vec<XdgDesktopEntry>>) {
		self.user_desktop_files = user_desktop_files;
	}
}