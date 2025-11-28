mod launcher;
mod state;

pub use launcher::{
    Launcher, focus_text_input, handle_enter_key, handle_result_box_hovered, hide_window,
    scroll_search_results_down,
};
pub use state::State;

pub const RESULT_ENTRY_COUNT: usize = 6;
