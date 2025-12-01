use std::cell::RefCell;
use std::collections::{HashMap, hash_map};
use std::rc::Rc;

use gtk::glib::{Object};
use gtk::prelude::{BoxExt, ButtonExt, LayoutManagerExt, WidgetExt};
use gtk::subclass::prelude::*;

use crate::gobject::{SearchResultBox, ScrollBar};
use crate::launcher;

mod inner {
    use super::*;

    pub struct SearchResultContainer {
    	pub result_boxes: RefCell<Vec<SearchResultBox>>,
    	pub inner: gtk::Box,
    	pub scroll_bar: ScrollBar,
    }

    #[gtk::glib::object_subclass]
    impl ObjectSubclass for SearchResultContainer {
        const NAME: &'static str = "SearchResultContainer";
        type Type = super::SearchResultContainer;
        type ParentType = gtk::Widget;

        fn new() -> Self {
        	let scroll_bar = ScrollBar::new();
        	scroll_bar.add_css_class("scroll-bar");
            scroll_bar.set_hexpand(false);
        	let inner = gtk::Box::new(gtk::Orientation::Vertical, 0);
            inner.set_hexpand(true);
            inner.add_css_class("result-list");
            Self {
            	result_boxes: Default::default(),
            	inner,
            	scroll_bar,
            }
        }
    }

    impl ObjectImpl for SearchResultContainer {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().set_layout_manager(Some(gtk::BinLayout::new()));
        }
    }
    impl WidgetImpl for SearchResultContainer {}
}

glib::wrapper! {
    pub struct SearchResultContainer(ObjectSubclass<inner::SearchResultContainer>)
    @extends gtk::Widget, gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl SearchResultContainer {
    pub fn new() -> Self {
        let obj = Object::new::<Self>();
        let result_box_container = &inner::SearchResultContainer::from_obj(&obj);

        let mut result_boxes = result_box_container.result_boxes.borrow_mut();

        for i in 0..launcher::RESULT_ENTRY_COUNT {
            let mut result_box = SearchResultBox::new(i);
            result_box.set_focusable(true);
            result_box.set_can_focus(true);
            result_box.set_focus_on_click(true);
            result_box.set_label(&"");
            result_box.add_css_class("result-box");
            result_box_container.inner.append(&result_box);
            result_boxes.push(result_box.into());
        }

        let container_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        container_box.append(&result_box_container.inner);
        container_box.append(&result_box_container.scroll_bar);
        container_box.set_parent(&obj);
        container_box.set_homogeneous(false);
        container_box.add_css_class("result-container");

        obj.set_child_visible(true);
        drop(result_boxes);
        obj
    }

    pub fn attach_result_box_handlers<T: Clone>(
            &self,
            attach_handlers: impl Fn(T, &SearchResultBox, usize),
            handler_cell_arg: T
        ) {
        for i in 0..launcher::RESULT_ENTRY_COUNT {
            attach_handlers(handler_cell_arg.clone(), &self.index(i), i);
        }
    }

    pub fn index(&self, idx: usize) -> SearchResultBox {
        let result_box_container = &inner::SearchResultContainer::from_obj(&self);
        result_box_container.result_boxes.borrow()[idx].clone()
    }

    pub fn hide(&self) {
        for i in 0..launcher::RESULT_ENTRY_COUNT {
            let result_box = self.index(i);
            // dresult_box.set_focusable(false);
            result_box.set_visible(false);
        }
    }

    pub fn len(&self) -> usize {
        let result_box_container = &inner::SearchResultContainer::from_obj(&self);
        result_box_container.result_boxes.borrow().len()
    }
}
