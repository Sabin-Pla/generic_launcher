


use gtk::prelude::FrameExt;
use std::borrow::BorrowMut;
use crate::XdgDesktopEntry;
use std::cell::RefCell;
use crate::Rc;
use std::ffi::OsStr;

type SearchResult = Vec<usize>;

const MAX_SEARCH_RESULTS: usize = 200;

#[derive(Default)]
pub struct SearchContext {
	pub user_desktop_files: Rc<Vec<XdgDesktopEntry>>,
	result_cache: SearchResult,
	pub buf: String
}


impl SearchContext {
	pub fn set_desktop_files(&mut self, user_desktop_files: Rc<Vec<XdgDesktopEntry>>) {
		self.user_desktop_files = user_desktop_files;
	}
}


pub fn get_xdg_desktop_entries() -> (Vec<XdgDesktopEntry>, Vec<XdgDesktopEntry>) {
	let home = std::env::var("HOME").unwrap_or("~".to_string());
	let dirs_entries = std::env::var("XDG_DATA_DIRS")
		.unwrap_or("/usr/local/share:/usr/share".to_string());
	let data_home = std::env::var("XDG_DATA_HOME")
		.unwrap_or(home + "/.local/share");
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
	let mut launcher_files: Vec<_> = applications_folders.filter_map(|folder| {
		println!("{:?}", folder);
		if !added.contains(&folder) { 
			// filter duplicates (if folder is in both env vars)
			added.push(folder.clone());
			return Some(std::path::Path::new(&folder).read_dir());
		}
		None
	}).collect();


	let mut custom_launcher = std::env::current_dir().expect("Error accessing CWD");
	custom_launcher.push("misc");
	launcher_files.push(custom_launcher.read_dir());

	let desktop_extension = Some(OsStr::new("desktop"));
	let mut entries: Vec<XdgDesktopEntry> = vec!(); 
	let mut custom_entries: Vec<XdgDesktopEntry> = vec!();

	for path in launcher_files {
		let path: std::fs::ReadDir = match path {
			Err(..) => continue,
			Ok(p) => p

		};
		let contents = path.map(|p| p.unwrap().path());
		for path in contents {
			if path.extension() == desktop_extension { 
				let entry = XdgDesktopEntry::try_from(&path);
				if let Some(entry) = entry {
					if let Some(bool_setting) = entry
							.app_info.locale_string("GenericLauncherCustom") {
						custom_entries.push(entry);
						continue
					}
					entries.push(entry);
					continue
				} 
				println!("could not create launcher for path {:?}", path);
			} 
		}
	}
	println!("{:#?}", entries);
	(entries, custom_entries)
}

fn get_search_score_for(entry: &XdgDesktopEntry, mut query_string: String) -> usize {
	let mut app_name = entry.display_name.clone();
	app_name.make_ascii_lowercase();
	query_string.make_ascii_lowercase();
	match app_name.find(&query_string) {
		Some(idx) => std::usize::MAX - idx,
		None => 0
	}
}

struct SearchCandidate {
	xdg_enties_idx: usize,
	score: usize
}


fn fetch_search_results(context: &SearchContext) -> SearchResult {
	let mut results: Vec<SearchCandidate> = Vec::with_capacity(context.user_desktop_files.len());
	for (idx, entry) in  context.user_desktop_files.iter().enumerate() {
		let score = get_search_score_for(entry, context.buf.clone());
		if score > 0 {
			results.push(SearchCandidate {
				score,
				xdg_enties_idx: idx 
			});
			if results.len() == MAX_SEARCH_RESULTS {
				break;
			}
		}
	}
	results.sort_by(|a, b| b.score.cmp(&a.score));
	let mut results: Vec<_> = results.iter().map(|candidate| candidate.xdg_enties_idx).collect();
	results
}

pub fn display_search_results(results: SearchResult) {
	unsafe {
		crate::launcher.clear_search_results();
		let mut counter = 0;
		for (idx, desktop_idx) in results.iter().enumerate() {
			if counter >= crate::RESULT_ENTRY_COUNT {
				break;
			}
			crate::launcher.set_search_frame(*desktop_idx, counter, idx);
			counter += 1;
		}
	}
}

pub fn text_inserted(context: &mut SearchContext, position: usize, chars: &str) {
	let mut buf = &mut (context.buf);
    buf.insert_str(position, chars);
    println!("buffer: {:#?}", &buf);
    let search_results = fetch_search_results(context);
    context.result_cache = search_results.clone();
    drop(context);
    display_search_results(search_results)

}

pub fn text_deleted(context: &mut SearchContext, position: usize, n_chars: Option<u32>) {
	if let Some(n) = n_chars {
		context.buf.drain(position..position+n as usize);
	} else {
		context.buf.drain(position..);
	}
	println!("buffer: {:#?}", &context.buf);
	let search_results = fetch_search_results(context);
	context.result_cache = search_results.clone();
	drop(context);
	display_search_results(search_results);
}

pub fn get_xdg_index_from_last_search_result_idx(context: &SearchContext, idx: usize) -> Option<usize> {
	context.result_cache.get(idx).copied()
}