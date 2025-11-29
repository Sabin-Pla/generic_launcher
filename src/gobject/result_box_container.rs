use std::cell::RefCell;
use std::collections::{HashMap, hash_map};
use std::rc::Rc;

use gtk::glib::{Object};
use gtk::prelude::{Cast, LayoutManagerExt, WidgetExt};
use gtk::subclass::prelude::*;

use crate::gobject::{SearchResultBox, ScrollBar};

mod inner {
    use super::*;

    pub struct ResultBoxContainer {
    	pub result_boxes: Vec<SearchResultBox>,
    	pub inner: gtk::Box,
    	pub scroll_bar: ScrollBar
    }

    #[gtk::glib::object_subclass]
    impl ObjectSubclass for ResultBoxContainer {
        const NAME: &'static str = "ResultBoxContainer";
        type Type = super::ResultBoxContainer;
        type ParentType = gtk::Widget;

        fn new() -> Self {
        	let scroll_bar = ScrollBar::new();
        	scroll_bar.add_css_class("scroll-bar'");
        	let inner = gtk::Box::new(gtk::Orientation::Vertical, 0);
        	inner.set_hexpand(true);
            Self {
            	result_boxes: Vec::new(),
            	inner,
            	scroll_bar
            }
        }
    }

    impl ObjectImpl for ResultBoxContainer {}
    impl WidgetImpl for ResultBoxContainer {}
}

glib::wrapper! {
    pub struct ResultBoxContainer(ObjectSubclass<inner::ResultBoxContainer>)
    @extends gtk::Widget, gtk::Box, gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl ResultBoxContainer {
    pub fn new() -> Self {
        let obj = Object::new::<Self>();
        let result_box_container = &inner::ResultBoxContainer::from_obj(&obj);
        let paned = gtk::Paned::builder()
    		.orientation(gtk::Orientation::Horizontal)
    		.start_child(&result_box_container.inner)
    		.end_child(&result_box_container.scroll_bar)
    		.build();
    	paned.set_parent(&obj);
        obj
    }
}
