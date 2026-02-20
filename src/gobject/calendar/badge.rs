use gtk::prelude::{WidgetExt};
use gtk::subclass::prelude::*;

/*
    Badge is used for dayboxes which have notes associated with them
*/

mod inner {
    use super::*;

    pub struct Badge {}

    #[gtk::glib::object_subclass]
    impl ObjectSubclass for Badge {
        const NAME: &'static str = "Badge";
        type Type = super::Badge;
        type ParentType = gtk::Widget;

        fn new() -> Self {
            Self {}
        }
    }

    impl ObjectImpl for Badge {}

    impl WidgetImpl for Badge {}
}

glib::wrapper! {
    pub struct Badge(ObjectSubclass<inner::Badge>)
    @extends gtk::Widget, gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl Badge {
    pub fn new() -> Self {
        let obj =  glib::Object::new::<Self>();
        obj.add_css_class("note-badge");
        obj.set_halign(gtk::Align::End);    
        obj.set_valign(gtk::Align::Start);
        obj.set_visible(false);
        obj
    }
}
