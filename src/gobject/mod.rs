mod clock_widget;
mod scroll_bar;
mod search_entry_buffer;
mod search_entry_im_context;
mod search_result_box;
mod result_box_container;

use scroll_bar::ScrollBar;

pub use clock_widget::ClockWidget;
pub use search_entry_buffer::SearchEntryBuffer;
pub use search_entry_im_context::SearchEntryIMContext;
pub use search_result_box::SearchResultBox;
pub use result_box_container::ResultBoxContainer;

// this module is used to handle all of the boiler-plate intensive code necessary to subclass structs as glib objects
