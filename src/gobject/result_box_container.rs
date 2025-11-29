use std::cell::RefCell;
use std::collections::{HashMap, hash_map};
use std::rc::Rc;

use gtk::glib::{Object};
use gtk::prelude::{Cast, LayoutManagerExt, WidgetExt};
use gtk::subclass::prelude::*;

use crate::gobject::{SearchResultBox, ScrollBar};
use crate::launcher;

mod inner {
    use super::*;

    pub struct ResultBoxContainer {
    	pub result_boxes: RefCell<Vec<SearchResultBox>>,
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
            	result_boxes: Default::default(),
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
    pub fn new<T: Clone>(
            attach_result_box_handlers: impl Fn(T, &mut SearchResultBox, usize),
            handler_cell_arg: T
        ) -> Self {
        let obj = Object::new::<Self>();
        let result_box_container = &inner::ResultBoxContainer::from_obj(&obj);
        let paned = gtk::Paned::builder()
    		.orientation(gtk::Orientation::Horizontal)
    		.start_child(&result_box_container.inner)
    		.end_child(&result_box_container.scroll_bar)
    		.build();
    	paned.set_parent(&obj);

        let mut result_boxes = result_box_container.result_boxes.borrow_mut();

        for i in 0..launcher::RESULT_ENTRY_COUNT {
            let mut result_box = SearchResultBox::new(i);
            result_box.set_focusable(true);
            result_box.set_can_focus(true);
            result_box.set_focus_on_click(true);
            gtk::prelude::ButtonExt::set_label(&result_box, &"");
            result_box.add_css_class("result-box");
            attach_result_box_handlers(handler_cell_arg.clone(), &mut result_box, i);
            result_boxes.push(result_box.into());
        }
        drop(result_boxes);
        obj
    }

    /*
    pub fn clear_result_boxes(&mut self) {
        let result_box_container = &inner::ResultBoxContainer::from_obj(&self);
        for result_box in &self.result_box_container.result_boxes.borrow_mut() {
            result_box.set_focusable(false);
            result_box.set_visible(false);
        }
    }*/
}
