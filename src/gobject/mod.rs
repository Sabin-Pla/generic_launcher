mod search_entry_im_context;
mod search_entry_buffer;
mod search_result_box;

pub use search_entry_buffer::SearchEntryBuffer;
pub use search_result_box::{SearchResultBox, SearchResultBoxWidget};
pub use search_entry_im_context::{SearchEntryIMContext};
// this module is used to handle all of the boiler-plate intensive code necessary to subclass structs as glib objects