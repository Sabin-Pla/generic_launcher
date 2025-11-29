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
            true
        }

        fn commit(&self, _: &str) {}

        fn preedit_start(&self) {}

        fn filter_keypress(&self, _event: &gdk::Event) -> bool {
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
        let obj = gtk::glib::Object::new::<Self>();
        obj
    }
}
