
use gtk::glib::{self, Object};
use gtk::subclass::prelude::*;
use gtk::subclass::prelude::DerivedObjectProperties;
use crate::{Arc, Mutex, Rc, RefCell};
use crate::search;
use crate::launcher::Launcher;

mod inner {
    use super::*;

    pub struct SearchEntryBuffer(pub RefCell<Option<Arc<Mutex<Launcher>>>>);

    #[gtk::glib::object_subclass]
    impl ObjectSubclass for SearchEntryBuffer {
        const NAME: &'static str = "SearchEntryBuffer";
        type Type = super::SearchEntryBuffer;
        type ParentType = gtk::EntryBuffer;

        fn new() -> Self {
            Self(RefCell::new(None))
        }
    }

    impl ObjectImpl for SearchEntryBuffer {}
    impl EntryBufferImpl for SearchEntryBuffer {
    	fn inserted_text(&self, position: u32, chars: &str) {
            let position = position as usize;
    		println!("text inserted at position {position}| {} {}", chars.len(), chars);
            if chars.len() == 1 && chars.as_bytes()[0] == 13 {
                return; // carrage return ascii, don't add control chars to buffer.
            }
            let mut launcher = self.0.borrow_mut();
            let mut launcher = launcher.as_mut().unwrap().lock().unwrap();
            search::text_inserted(&mut launcher, position, chars);
    	}

        fn text(&self) -> glib::GString {
            println!("SearchEntryBuffer text()");
            let mut launcher = self.0.borrow_mut();
            let mut launcher = launcher.as_mut().unwrap().lock().unwrap();
            let context = &launcher.search_context;
            glib::GString::from_string_unchecked(context.buf.to_string())
        }

        fn deleted_text(&self, position: u32, n_chars: Option<u32>) { 
            println!("SearchEntryBuffer deleted_text()");
            let position = position as usize;
            let mut launcher = self.0.borrow_mut();
            let mut launcher = launcher.as_mut().unwrap().lock().unwrap();
            search::text_deleted(&mut launcher, position, n_chars);
            println!("deleted text: {position}");
        }

        fn length(&self) -> u32 {
            println!("SearchEntryBuffer length()");
            let mut launcher = self.0.borrow_mut();
            let mut launcher = launcher.as_mut().unwrap().lock().unwrap();
            launcher.search_context.buf.chars().count().try_into().unwrap()
        } 
    }


}

glib::wrapper! {
    pub struct SearchEntryBuffer(ObjectSubclass<inner::SearchEntryBuffer>)  
    @extends gtk::EntryBuffer; 
}

impl SearchEntryBuffer {
    pub fn new(launcher_arc: Arc<Mutex<Launcher>>) -> Self {
        use std::borrow::Borrow;
        let mut obj = Object::new::<Self>();
        *inner::SearchEntryBuffer::from_obj(&obj).0.borrow_mut() = Some(launcher_arc);
        obj
    }
}
