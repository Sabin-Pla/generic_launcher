


use crate::XdgDesktopEntry;
use std::cell::RefCell;
use crate::Rc;
use std::ffi::OsStr;

type SearchResult = Vec<usize>;

#[derive(Default)]
pub struct SearchContext {
	user_desktop_files: Vec<XdgDesktopEntry>,
	result_cache: Vec<SearchResult>,
	buf: Rc<String>
}


impl SearchContext {
	fn last_search_results(&mut self) -> Option<SearchResult> {
		match self.result_cache.last() {
			Some(result) => Some(result.clone()),
			None => None,
		}
	}

	fn refine_search_results(&mut self) {
		
	}

	pub fn set_desktop_files(&mut self, user_desktop_files: Vec<XdgDesktopEntry>) {
		self.user_desktop_files = user_desktop_files;
	}
}


pub fn get_xdg_desktop_entries() -> Vec<XdgDesktopEntry> {
	let data_home =std::env::var("XDG_DATA_HOME")
		.unwrap_or("/usr/local/share:/usr/share".to_string());
	let dirs_entries = std::env::var("XDG_DATA_DIRS")
		.unwrap_or("/.local/share".to_string());
 	let applications_folders = [data_home.split(':'), dirs_entries.split(':')]
		.into_iter()
		.flatten()
		.map(|mut d| {
			let mut d2 = d.chars();
			if let Some('/') = d2.next_back() {
				d = d2.as_str(); // remove ending '/' if present
			}
			d.to_owned() + "/applications"
		});

	let mut added = vec!();
	let launcher_files = applications_folders.filter_map(|folder| {
		if !added.contains(&folder) { 
			// filter duplicates (if folder is in both env vars)
			added.push(folder.clone());
			return Some(std::path::Path::new(&folder).read_dir());
		}
		None
	});

	let desktop_extension = Some(OsStr::new("desktop"));
	let mut entries: Vec<XdgDesktopEntry> = vec!(); 

	for path in launcher_files {
		let path: std::fs::ReadDir = match path {
			Err(..) => continue,
			Ok(p) => p

		};
		let contents = path.map(|p| p.unwrap().path());
		for path in contents {
			if path.extension() == desktop_extension { 
				let entry = XdgDesktopEntry::try_from(&path);
				if entry.is_some() {
					entries.push(entry.unwrap());
					continue
				} 
				println!("could not create launcher for path {:?}", path);
			} 
		}
	}
	entries
}

pub fn refetch_search_results(context: &mut SearchContext) -> Option<SearchResult> {
	context.refine_search_results();
	context.last_search_results()
}

pub fn chars_removed_from_buffer_end(
	context: &mut SearchContext, 
	num_chars: usize) -> SearchResult {
	
	todo!()
}

pub fn chars_added_to_buffer_end(
	context: &mut SearchContext,
	idx: usize) -> SearchResult {

	todo!()
}

pub fn chars_were_inserted(context: &mut SearchContext) {
	// for insertions into search buffer, just throw out the cache
}