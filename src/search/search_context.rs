use crate::search::SearchResult;
use crate::xdg_desktop_entry::XdgDesktopEntry;
use crate::{Rc, Vec};

#[derive(Default)]
pub struct SearchContext {
    pub user_desktop_files: Rc<Vec<XdgDesktopEntry>>,
    pub(super) result_cache: SearchResult,
}
