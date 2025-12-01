use gtk::glib::{Object};
use gtk::subclass::prelude::*;
use gtk::prelude::WidgetExt;

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

    impl ObjectImpl for ScrollBar {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().set_layout_manager(Some(gtk::BinLayout::new()));
        }
    }
    impl WidgetImpl for ScrollBar {}
}

glib::wrapper! {
    pub struct ScrollBar(ObjectSubclass<inner::ScrollBar>)
    @extends gtk::Widget, gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl ScrollBar {
    pub fn new() -> Self {
        let obj = Object::new::<Self>();
        obj.set_hexpand(true);
        obj.set_vexpand(true);
        obj
    }
}
