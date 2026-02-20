use std::cell::RefCell;

use gtk::glib::{Object};
use gtk::prelude::{AdjustmentExt, BoxExt, ButtonExt, WidgetExt};
use gtk::subclass::prelude::*;

use crate::gobject::SearchResultBox;
use crate::launcher;

mod inner {
    use super::*;

    pub struct SearchResultContainer {
    	pub result_boxes: RefCell<Vec<SearchResultBox>>,
    	pub inner: gtk::Box,
    	pub scroll_bar: gtk::Scrollbar
    }

    #[gtk::glib::object_subclass]
    impl ObjectSubclass for SearchResultContainer {
        const NAME: &'static str = "SearchResultContainer";
        type Type = super::SearchResultContainer;
        type ParentType = gtk::Widget;

        fn new() -> Self {
        	let scroll_bar = gtk::Scrollbar::new(gtk::Orientation::Vertical, None::<&gtk::Adjustment>);
        	scroll_bar.add_css_class("scroll-bar");
            scroll_bar.set_hexpand(false);
            scroll_bar.adjustment().set_page_increment(1.0);
        	let inner = gtk::Box::new(gtk::Orientation::Vertical, 0);
            inner.set_hexpand(true);
            inner.add_css_class("result-list");
            Self {
            	result_boxes: Default::default(),
            	inner,
            	scroll_bar
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
            let result_box = SearchResultBox::new(i);
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

        let scroll = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
        let adjustment = result_box_container.scroll_bar.adjustment();
        scroll.connect_scroll(move |_, _dx, dy| {
            adjustment.set_value(adjustment.value() + dy);
            glib::Propagation::Proceed
        });
        obj.add_controller(scroll);

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

    pub fn attach_scroll_bar_handler<T: Clone + 'static>(
            &self,
            handler: impl Fn(T, &gtk::Adjustment) + 'static,
            handler_cell_arg: T
        ) {
        let result_box_container = &inner::SearchResultContainer::from_obj(&self);
        let handler_wrapper = move |adjustment:  &gtk::Adjustment| {
            handler(handler_cell_arg.clone(), adjustment)
        };
        result_box_container.scroll_bar.adjustment().connect_value_changed(handler_wrapper);
    }

    pub fn index(&self, idx: usize) -> SearchResultBox {
        let result_box_container = &inner::SearchResultContainer::from_obj(&self);
        result_box_container.result_boxes.borrow()[idx].clone()
    }

    pub fn hide(&self, indexes: Option<std::ops::Range<usize>>) {
        match indexes {
            Some(range) => {
                for i in range {
                    self.index(i).set_visible(false);
                }
            }
            None => self.set_visible(false)
        }
    }

    pub fn show(&self) {
        self.set_visible(true);
    }

    pub fn len(&self) -> usize {
        let result_box_container = &inner::SearchResultContainer::from_obj(&self);
        result_box_container.result_boxes.borrow().len()
    }

    pub fn adjust_scrollbar(&self, top_idx: usize, result_count: usize, per_page: usize) {
        let result_box_container = &inner::SearchResultContainer::from_obj(&self);
        let scroll_bar = &result_box_container.scroll_bar;
        let value = top_idx as f64;
        let lower = 0.0;
        let upper = result_count as f64;
        let page_size = per_page as f64;
        let adjustment = scroll_bar.adjustment();
        adjustment.set_upper(upper);
        adjustment.set_lower(lower);
        adjustment.set_value(value);
        adjustment.set_page_size(page_size);
    }
}

