pub mod clock;
mod launcher;
mod state;

pub use launcher::Launcher;
pub use state::State;

pub const RESULT_ENTRY_COUNT: usize = 6;

pub static mut LAUNCHER: Launcher = Launcher { 
	state: State::NotStarted, 
	done_init: false,
    css_provider: None,
    fifo_path: ['\0' as i8; 2000],
    search_result_frames: vec!(),
    selected_search_idx: None,
    text_input: None,
    user_desktop_files: None,
    search_context: None,
    input_buffer: None,
    custom_launchers: None,
    screenshot_button: None,
    hovered_idx: 0,
    clock: None,
    clock_sizes: None,
    current_monitor: None,
}; 