use super::*;
use crate::xdg_desktop_entry::XdgDesktopEntry;
use crate::utils;
use crate::launcher;
use crate::launcher::{RESULT_ENTRY_COUNT, Launcher};

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
	let results: Vec<_> = results.iter().map(|candidate| candidate.xdg_enties_idx).collect();
	results
}

pub fn display_search_results(launcher: &mut Launcher, results: SearchResult) {
	launcher.clear_search_results();
	let mut counter = 0;
	for (idx, desktop_idx) in results.iter().enumerate() {
		if counter >= RESULT_ENTRY_COUNT {
			break;
		}
		launcher.set_search_frame(*desktop_idx, counter, idx);
		counter += 1;
	}
}

pub fn text_inserted(launcher: &mut Launcher, position: usize, chars: &str) {
	let buf = &mut (launcher.search_context.buf);
	// buffer is not garuanteed to be full of utf8 characters, so we can't just
	// insert the char at the given position 
    buf.insert_str(utils::char_position(buf, position), chars);
    let search_results = fetch_search_results(&launcher.search_context);
    launcher.search_context.result_cache = search_results.clone();
    display_search_results(launcher, search_results)
}

pub fn text_deleted(launcher: &mut Launcher, position: usize, n_chars: Option<u32>) {
	// position is one less than the number of chars after which the cursor is placed
	// n_chars is Some(1) when 
	let position_idx = utils::char_position(&launcher.search_context.buf, position);

	if let Some(n) = n_chars {
		let end_idx = utils::char_position(&launcher.search_context.buf[position_idx..], n as usize);
		println!("Draining {} {position_idx}..{end_idx} {n}", &launcher.search_context.buf);
		&launcher.search_context.buf.drain(position_idx..position_idx+end_idx);
	} else {
		&launcher.search_context.buf.drain(position_idx..);
	}

	
	if let launcher::State::Hidden = launcher.state {	
		return
	}
	
	let search_results = fetch_search_results(&launcher.search_context);
	launcher.search_context.result_cache = search_results.clone();
	display_search_results(launcher, search_results);
}

pub fn get_xdg_index_from_last_search_result_idx(context: &SearchContext, idx: usize) -> Option<usize> {
	context.result_cache.get(idx).copied()
}