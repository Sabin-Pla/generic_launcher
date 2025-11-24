use crate::Rc;
use gtk::glib::{self, Object};
use gtk::subclass::prelude::*;
use gtk::prelude::*;
use gtk::IMContext;
use gtk::EventControllerKey;
use gtk::prelude::IMContextExt;
use std::cell::{Ref, RefCell};
use crate::search::SearchContext;
use crate::search;

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
            search::text_deleted(&mut context, position, n_chars.clone());
            println!("deleted text: {position} {:?}", &n_chars);
        }

        fn length(&self) -> u32 {
            self.0.borrow_mut().context.borrow_mut().buf.chars().count().try_into().unwrap()
        } 
    }

    
    pub struct SearchEntryIMContext();

    #[gtk::glib::object_subclass]
    impl ObjectSubclass for SearchEntryIMContext {
        const NAME: &'static str = "SearchEntryIMContext";
        type Type = super::SearchEntryIMContext;
        type ParentType = IMContext;

        fn new() -> Self {
            Self()
        }
    }

    impl ObjectImpl for SearchEntryIMContext {}
    impl IMContextImpl for SearchEntryIMContext {
         fn retrieve_surrounding(&self) -> bool {
            println!("?????");
           /** let me = self.0.borrow_mut();
            let context = me.context.borrow_mut();
            let buffer =  &context.buf;
            let obj = self.obj();
           // obj.set_use_preedit(false);
            obj.set_surrounding_with_selection("", 0, 0); */

            true
         }

         fn commit(&self, _: &str) {
            println!("commit");
         }

         fn preedit_start(&self) {
            println!("PREDIT START");
         }

          fn filter_keypress(&self, event: &gdk::Event) -> bool {
                // Print something easy to spot
                let (preedit, cursor_pos, idx) = self.preedit_string();
                println!("IM: filter_keypress {:?}", (preedit, cursor_pos, idx));

                // Claim the event so GTK will call the other IM methods
                false
            }
    }

}

glib::wrapper! {
    pub struct SearchEntry(ObjectSubclass<inner::SearchEntry>)  
    @extends gtk::Widget, gtk::EntryBuffer; 
}

impl SearchEntry {
    pub fn new(data: SearchEntryBuffer) -> Self {
        let obj = Object::new::<Self>();
        *inner::SearchEntry::from_obj(&obj).0.borrow_mut() = data;
        obj
    }

   /* pub fn get(&self) -> Ref<Self> {
        inner::SearchEntry::from_obj(self).0.borrow()
    }*/
}

glib::wrapper! {
    pub struct SearchEntryIMContext(ObjectSubclass<inner::SearchEntryIMContext>)  
    @extends gtk::IMContext;
}

impl SearchEntryIMContext {
    pub fn new() -> Self {
        let obj = Object::new::<Self>();
       // inner::SearchEntryIMContext::from_obj(&obj);
        obj
    }
}