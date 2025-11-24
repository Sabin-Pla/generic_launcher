
use gtk::glib::{self, Object};
use gtk::subclass::prelude::*;
use gtk::subclass::prelude::DerivedObjectProperties;
use crate::{Arc, Mutex, Rc, RefCell};
use crate::search;
use crate::utils;
use crate::launcher::Launcher;

mod inner {
    use super::*;

    pub struct SearchEntryBuffer(pub Rc<RefCell<String>>);

    #[gtk::glib::object_subclass]
    impl ObjectSubclass for SearchEntryBuffer {
        const NAME: &'static str = "SearchEntryBuffer";
        type Type = super::SearchEntryBuffer;
        type ParentType = gtk::EntryBuffer;

        fn new() -> Self {
            Self(Rc::new(RefCell::new("".to_string())))
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
            let mut buf = self.0.borrow_mut();
            // buffer is not garuanteed to be full of utf8 characters, so we can't just
            // insert the char at the given position 
            let position = utils::char_position(&buf, position);
            buf.insert_str(position, chars);
    	}

        fn text(&self) -> glib::GString {
            println!("SearchEntryBuffer text()");
            glib::GString::from_string_unchecked(self.0.borrow().clone())
        }

        fn deleted_text(&self, position: u32, n_chars: Option<u32>) { 
            println!("SearchEntryBuffer deleted_text()");
            let position = position as usize;
            let mut buf = self.0.borrow_mut();
            let position_idx = utils::char_position(&buf, position);

            if let Some(n) = n_chars {
                let end_idx = utils::char_position(&buf[position_idx..], n as usize);
                println!("Draining {} {position_idx}..{end_idx} {n}", &buf);
                &buf.drain(position_idx..position_idx+end_idx);
            } else {
                &buf.drain(position_idx..);
            }
            println!("deleted text: {position}");
        }

        fn length(&self) -> u32 {
            println!("SearchEntryBuffer length()");
            self.0.borrow_mut().chars().count().try_into().unwrap()
        } 
    }


}

glib::wrapper! {
    pub struct SearchEntryBuffer(ObjectSubclass<inner::SearchEntryBuffer>)  
    @extends gtk::EntryBuffer; 
}

use std::cell::Ref;
impl SearchEntryBuffer {
    pub fn new() -> Self {
        Object::new::<Self>()
    }

    pub fn length(&self) -> u32 {
        inner::SearchEntryBuffer::from_obj(self).length()
    }

    pub fn text(&self) -> Rc<RefCell<String>> {
        inner::SearchEntryBuffer::from_obj(self).0.clone()
    }
}
