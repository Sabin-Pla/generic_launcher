use std::cell::RefCell;
use std::collections::{HashMap, hash_map};
use std::rc::Rc;

use gtk::glib::{Object};
use gtk::prelude::{Cast, LayoutManagerExt, WidgetExt};
use gtk::subclass::prelude::*;

mod inner {
    use super::*;

    pub struct ResultBoxContainer();

    #[gtk::glib::object_subclass]
    impl ObjectSubclass for ResultBoxContainer {
        const NAME: &'static str = "ResultBoxContainer";
        type Type = super::ResultBoxContainer;
        type ParentType = gtk::Widget;

        fn new() -> Self {
            Self()
        }
    }

    impl ObjectImpl for ResultBoxContainer {
    	fn constructed(&self) {
            self.parent_constructed();

            // Give this widget a layout manager
            let obj = self.obj();
            obj.set_layout_manager(Some(gtk::BoxLayout::new(gtk::Orientation::Vertical)));
        }
    }
    impl WidgetImpl for ResultBoxContainer {}
}

glib::wrapper! {
    pub struct ResultBoxContainer(ObjectSubclass<inner::ResultBoxContainer>)
    @extends gtk::Widget, gtk::Box, gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}
