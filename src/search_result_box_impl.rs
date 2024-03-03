use crate::Rc;
use std::ffi::OsString;
use std::error::Error;
use gtk::glib::{self, Object};
use gtk::subclass::prelude::*;
use std::cell::{Ref, RefCell};
use crate::launcher;

pub struct SearchResultBoxWidget { 
	pub idx_in_container: usize,
    pub idx_in_xdg_entries_vector: Rc<RefCell<usize>>,
    pub idx_in_search_result_vector: Rc<RefCell<usize>>
}

impl SearchResultBoxWidget {
	pub fn new() -> Self { 
		Self { 
			idx_in_container: 0,
            idx_in_xdg_entries_vector: Rc::new(0.into()),
            idx_in_search_result_vector: Rc::new(0.into())
		} 
	}

	pub fn from(idx: usize) -> Self {
		Self { 
            idx_in_container: idx, 
            idx_in_xdg_entries_vector: Rc::new(0.into()),
            idx_in_search_result_vector: Rc::new(0.into())
        }
	}

    pub fn set_idx_in_xdg_entries_vector(&self, idx: usize) {
        let mut rc = self.idx_in_xdg_entries_vector.borrow_mut();
        *rc = idx;
    } 

    pub fn set_idx_in_search_result_vector(&self, idx: usize) {
        let mut rc = self.idx_in_search_result_vector.borrow_mut();
        *rc = idx;
    } 
}

mod inner {
    use super::*;

    pub struct SearchResultBox(pub RefCell<SearchResultBoxWidget>);


    #[gtk::glib::object_subclass]
    impl ObjectSubclass for SearchResultBox {
        const NAME: &'static str = "SearchResultBox";
        type Type = super::SearchResultBox;
        type ParentType = gtk::Frame;

        fn new() -> Self {
            Self(SearchResultBoxWidget::new().into())
        }
    }

    impl ObjectImpl for SearchResultBox {}
    impl WidgetImpl for SearchResultBox {}
    impl FrameImpl for SearchResultBox {}
}

glib::wrapper! {
    pub struct SearchResultBox(ObjectSubclass<inner::SearchResultBox>)
    @extends gtk::Widget, gtk::Frame;
}

impl SearchResultBox {
    pub fn new(data: SearchResultBoxWidget) -> Self {
        let obj = Object::new::<Self>();
        *inner::SearchResultBox::from_instance(&obj).0.borrow_mut() = data;
        obj
    }

    pub fn get(&self) -> Ref<SearchResultBoxWidget> {
        inner::SearchResultBox::from_instance(self).0.borrow()
    }

    pub fn set_desktop_idx(&mut self, idx: usize) {
        let mut inner = self.get();
        inner.set_idx_in_xdg_entries_vector(idx);
    } 

    pub fn get_desktop_idx(&self) -> usize {
        let mut inner = self.get();
        let val = inner.idx_in_xdg_entries_vector.borrow_mut();
        val.clone()
    }

    pub fn set_idx_in_search_result_vector(&mut self, idx: usize) {
        let mut inner = self.get();
        inner.set_idx_in_search_result_vector(idx);
    }

    pub fn get_idx_in_search_result_vector(&self) -> usize {
        let inner = self.get();
        let val = inner.idx_in_search_result_vector.borrow_mut();
        val.clone()
    }
}