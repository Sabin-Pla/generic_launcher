use crate::{Rc, RefCell, search::SearchContext};

pub struct SearchEntryBuffer { 
	pub context: Rc<RefCell<SearchContext>>
}

impl SearchEntryBuffer {
	pub fn new() -> Self { 
		Self { 
			context: Rc::default()
		} 
	}
}