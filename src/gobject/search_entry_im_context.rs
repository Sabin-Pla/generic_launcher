use gtk::glib::{self, Object};
use gtk::subclass::prelude::*;

mod inner {
    use super::*;
    
    pub struct SearchEntryIMContext();

    #[gtk::glib::object_subclass]
    impl ObjectSubclass for SearchEntryIMContext {
        const NAME: &'static str = "SearchEntryIMContext";
        type Type = super::SearchEntryIMContext;
        type ParentType = gtk::IMContext;

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

          fn filter_keypress(&self, _event: &gdk::Event) -> bool {
                // Print something easy to spot
                let (preedit, cursor_pos, idx) = self.preedit_string();
                println!("IM: filter_keypress {:?}", (preedit, cursor_pos, idx));

                // Claim the event so GTK will call the other IM methods
                false
            }
    }

}

glib::wrapper! {
    pub struct SearchEntryIMContext(ObjectSubclass<inner::SearchEntryIMContext>)  
    @extends gtk::IMContext;
}

impl SearchEntryIMContext {
    pub fn new() -> Self {
        let obj = Object::new::<Self>();
        obj
    }
}