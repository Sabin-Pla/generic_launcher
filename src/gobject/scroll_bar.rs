use gtk::glib::{Object};
use gtk::subclass::prelude::*;

mod inner {
    use super::*;

    pub struct ScrollBar();

    #[gtk::glib::object_subclass]
    impl ObjectSubclass for ScrollBar {
        const NAME: &'static str = "ScrollBar";
        type Type = super::ScrollBar;
        type ParentType = gtk::Widget;

        fn new() -> Self {
            Self()
        }
    }

    impl ObjectImpl for ScrollBar {}
    impl WidgetImpl for ScrollBar {}
}

glib::wrapper! {
    pub struct ScrollBar(ObjectSubclass<inner::ScrollBar>)
    @extends gtk::Widget, gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl ScrollBar {
    pub fn new() -> Self {
        Object::new::<Self>()
    }
}
