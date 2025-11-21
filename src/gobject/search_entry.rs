use gtk::glib::{self, Object};
use gtk::subclass::prelude::*;

use crate::{Rc, RefCell};
use crate::search;

use super::SearchEntryBuffer;

mod inner {
    use super::*;

    pub struct SearchEntry(pub Rc<RefCell<SearchEntryBuffer>>);

    #[gtk::glib::object_subclass]
    impl ObjectSubclass for SearchEntry {
        const NAME: &'static str = "SearchEntry";
        type Type = super::SearchEntry;
        type ParentType = gtk::EntryBuffer;

        fn new() -> Self {
            let buffer = Rc::new(RefCell::new(SearchEntryBuffer::new()));
            let me = Self(buffer.clone());
            me
        }
    }

    impl ObjectImpl for SearchEntry {}
    impl EntryBufferImpl for SearchEntry {
    	fn inserted_text(&self, position: u32, chars: &str) {
            let position = position as usize;
    		println!("text inserted at position {position}| {} {}", chars.len(), chars);
            if chars.len() == 1 && chars.as_bytes()[0] == 13 {
                return; // carrage return ascii, don't add control chars to buffer.
            }
            let me = self.0.borrow_mut();
            search::text_inserted(&mut me.context.borrow_mut(), position, chars);
    	}

        fn text(&self) -> glib::GString {
            let me = self.0.borrow_mut();
            let context = me.context.borrow_mut();
            let buffer =  &context.buf;
            glib::GString::from_string_unchecked(buffer.to_string())
        }

        fn deleted_text(&self, position: u32, n_chars: Option<u32>) { 
            let me = self.0.borrow_mut();
            let position = position as usize;
            let mut context = me.context.borrow_mut();
            search::text_deleted(&mut context, position, n_chars);
            println!("deleted text: {position}");
        }

        fn length(&self) -> u32 {
            self.0.borrow_mut().context.borrow_mut().buf.chars().count().try_into().unwrap()
        } 
    }


}

glib::wrapper! {
    pub struct SearchEntry(ObjectSubclass<inner::SearchEntry>)  
    @extends gtk::EntryBuffer; 
}

impl SearchEntry {
    pub fn new(data: SearchEntryBuffer) -> Self {
        let obj = Object::new::<Self>();
        *inner::SearchEntry::from_obj(&obj).0.borrow_mut() = data;
        obj
    }
}
