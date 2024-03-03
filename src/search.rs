


use gtk::prelude::FrameExt;
use std::borrow::BorrowMut;
use crate::XdgDesktopEntry;
use std::cell::RefCell;
use crate::Rc;
use std::ffi::OsStr;

type SearchResult = Vec<usize>;

#[derive(Default)]
pub struct SearchContext {
	user_desktop_files: Vec<XdgDesktopEntry>,
	result_cache: Vec<SearchResult>,
	pub buf: String
}


impl SearchContext {
	pub fn set_desktop_files(&mut self, user_desktop_files: Vec<XdgDesktopEntry>) {
		self.user_desktop_files = user_desktop_files;
	}
}


pub fn get_xdg_desktop_entries() -> Vec<XdgDesktopEntry> {
	let data_home = std::env::var("XDG_DATA_HOME")
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

fn get_search_score_for(entry: &XdgDesktopEntry, query_string: String) -> usize {
	let app_name = &entry.display_name;
	let name_length = app_name.len() as f64;
	match app_name.find(&query_string) {
		Some(idx) => (query_string.len() as f64 * (name_length - idx as f64) /  name_length) as usize,
		None => 0
	}
}

#[derive(Eq, PartialEq, PartialOrd)]
struct SearchCandidate {
	xdg_enties_idx: usize,
	score: usize
}

impl std::cmp::Ord for SearchCandidate {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.score.cmp(&other.score)
	}
}

fn fetch_search_results(context: &mut SearchContext) -> SearchResult {
	let mut results: Vec<SearchCandidate> = Vec::with_capacity(context.user_desktop_files.len());
	for (idx, entry) in  context.user_desktop_files.iter().enumerate() {
		println!("{:?}", entry);
		let score = get_search_score_for(entry, context.buf.clone());
		if score > 0 {
			results.push(SearchCandidate {
				score,
				xdg_enties_idx: idx 
			});
			if results.len() == crate::RESULT_ENTRY_COUNT {
				break;
			}
		}
	}
	results.sort();
	let mut results: Vec<_> = results.iter().map(|candidate| candidate.xdg_enties_idx).collect();
	println!("results");
	results
}

pub fn display_search_results(context: &mut SearchContext, results: SearchResult) {
	for (counter, idx) in results.iter().enumerate().rev() {
		unsafe {
			crate::launcher.search_result_frames[counter].set_label(
				Some(&context.user_desktop_files[*idx].display_name));
		}
	}
}

pub fn text_inserted(context: &mut SearchContext, position: usize, chars: &str) {
	let mut buf = &mut (context.buf);
    buf.insert_str(position, chars);
    println!("buffer: {:#?}", &buf);
    let search_results = fetch_search_results(context);
    display_search_results(context, search_results)

}

pub fn text_deleted(context: &mut SearchContext, position: usize, n_chars: Option<u32>) {
	if let Some(n) = n_chars {
		context.buf.drain(position..position+n as usize);
	} else {
		context.buf.drain(position..);
	}
	println!("buffer: {:#?}", &context.buf);
	let search_results = fetch_search_results(context);
	display_search_results(context, search_results);
}