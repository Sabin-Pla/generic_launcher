mod calendar;
mod centered_widget;
mod clock_widget;
mod search_entry_buffer;
mod search_entry_im_context;
mod search_result_box;
mod search_result_container;
mod volume_control;

pub(crate) use centered_widget::CenteredWidget;
pub use calendar::Calendar;
pub use clock_widget::ClockWidget;
pub use search_entry_buffer::SearchEntryBuffer;
pub use search_entry_im_context::SearchEntryIMContext;
pub use search_result_box::SearchResultBox;
pub use search_result_container::SearchResultContainer;
pub use volume_control::VolumeControl;

// this module is used to handle all of the boiler-plate intensive code necessary to subclass structs as glib objects
