


use std::cell::RefCell;
use crate::Rc;


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

pub struct XdgDesktopEntry {
	display_name: String,
	names: Vec<String>,
	keywords: Vec<String>
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


	let launcher_files = applications_folders.map(|folder| {
		println!("f {}", folder);
		std::path::Path::new(&folder).read_dir()
	});
	for path in launcher_files {
		println!("path {:?}", path);
		let path: std::fs::ReadDir = match path {
			Err(..) => continue,
			Ok(p) =>  p
		};
		let contents = path.map(|p| {
				let p = p.unwrap();
				(	
					p.path(),//.into_string().expect(
			//			&format!("cannot read file with name {:?}", p.file_name())), 
					std::fs::read_to_string(p.path())
				)
			}
		);
		for (path, contents) in contents {
			println!("PATH +================");
			println!("{:?}", path);
			println!("{}", contents.unwrap());
		}
		//let contents = std::fs::read_to_string(path.next());
	}
	todo!("check extra : + /");
	todo!()
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
	todo!("thread this")
}