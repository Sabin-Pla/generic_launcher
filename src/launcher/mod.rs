pub mod clock;
mod launcher;
mod state;

pub use launcher::{Launcher, 
	hide_window, handle_enter_key, focus_text_input, 
	scroll_search_results_down, handle_result_box_hovered};
pub use state::State;

pub const RESULT_ENTRY_COUNT: usize = 6;
