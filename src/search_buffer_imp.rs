use crate::Rc;
use std::ffi::OsString;
use std::error::Error;
use gtk::glib::{self, Object};
use gtk::subclass::prelude::*;
use std::cell::{Ref, RefCell};
use std::time::Duration;
use crate::search::SearchContext;
use crate::search;

pub struct SearchEntryBuffer { 
	pub context: Rc<RefCell<SearchContext>>,
	buf: Rc<RefCell<String>>,
}

impl SearchEntryBuffer {
	pub fn new() -> Self { 
		Self { 
			context: Rc::default(), 
			buf: Rc::default() 
		} 
	}
}

mod inner {
	use super::*;

    pub struct SearchEntry(pub RefCell<SearchEntryBuffer>);


    #[gtk::glib::object_subclass]
    impl ObjectSubclass for SearchEntry {
        const NAME: &'static str = "SearchEntry";
        type Type = super::SearchEntry;
        type ParentType = gtk::EntryBuffer;

        fn new() -> Self {
            Self(SearchEntryBuffer::new().into())
        }
    }

    impl ObjectImpl for SearchEntry {}
    impl EntryBufferImpl for SearchEntry {
    	fn inserted_text(&self, position: u32, chars: &str) {
    		println!("text inserted at position {position}");
    		println!("\"{chars}\"");
    		// let entry_buffer = self.0.borrow_mut();
    		let mut obj = self.0.borrow_mut();
    		*obj.buf.borrow_mut() += chars;
    		search::refetch_search_results(&mut obj.context.borrow_mut());

    	}
    }
}

glib::wrapper! {
    pub struct SearchEntry(ObjectSubclass<inner::SearchEntry>)
    @extends gtk::Widget, gtk::EntryBuffer, gtk::SearchEntry;
}

impl SearchEntry {
    pub fn new(data: SearchEntryBuffer) -> Self {
        let obj = Object::new::<Self>();
        *inner::SearchEntry::from_instance(&obj).0.borrow_mut() = data;
        obj
    }

    pub fn get(&self) -> Ref<SearchEntryBuffer> {
        inner::SearchEntry::from_instance(self).0.borrow()
    }
}