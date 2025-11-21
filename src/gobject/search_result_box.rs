use std::cell::RefMut;
use std::cell::{Ref, RefCell};

use gtk::glib::{self, Object};
use gtk::ConstraintTarget;
use gtk::subclass::prelude::*;
use gtk::Buildable;
use gtk::Accessible;
use gtk::Actionable;

pub struct SearchResultBoxWidget { 
	pub idx_in_container: usize,
    pub idx_in_xdg_entries_vector: usize,
    pub idx_in_search_result_vector: usize,
}

impl SearchResultBoxWidget {
	pub fn new() -> Self { 
		Self { 
			idx_in_container: 0,
            idx_in_xdg_entries_vector: 0,
            idx_in_search_result_vector: 0,
		} 
	}

	pub fn from(idx: usize) -> Self {
		Self { 
            idx_in_container: idx, 
            idx_in_xdg_entries_vector: 0,
            idx_in_search_result_vector: 0
        }
	}

    pub fn set_idx_in_xdg_entries_vector(&mut self, idx: usize) {
        self.idx_in_xdg_entries_vector = idx;
    } 

    pub fn set_idx_in_search_result_vector(&mut self, idx: usize) {
        self.idx_in_search_result_vector = idx;
    } 
}

mod inner {
    use super::*;

    pub struct SearchResultBox(pub RefCell<SearchResultBoxWidget>);

    impl ObjectImpl for SearchResultBox {}
    impl WidgetImpl for SearchResultBox {}
    impl ButtonImpl for SearchResultBox {}

    #[gtk::glib::object_subclass]
    impl ObjectSubclass for SearchResultBox {
        const NAME: &'static str = "SearchResultBox";
        type Type = super::SearchResultBox;
        type ParentType = gtk::Button;

        fn new() -> Self {
            Self(SearchResultBoxWidget::new().into())
        }
    }
}

glib::wrapper! {
    pub struct SearchResultBox(ObjectSubclass<inner::SearchResultBox>)
    @extends gtk::Widget, gtk::Button, gtk::Frame, ConstraintTarget, Buildable, Accessible, Actionable; 
}

impl SearchResultBox {
    pub fn new(data: SearchResultBoxWidget) -> Self {
        let obj = Object::new::<Self>();
        *inner::SearchResultBox::from_obj(&obj).0.borrow_mut() = data;
        obj
    }

    pub fn get(&self) -> Ref<SearchResultBoxWidget> {
        inner::SearchResultBox::from_obj(self).0.borrow()
    }

    pub fn get_mut(&self) -> RefMut<SearchResultBoxWidget> {
        inner::SearchResultBox::from_obj(self).0.borrow_mut()
    }

    pub fn set_desktop_idx(&mut self, idx: usize) {
        let mut inner = self.get_mut();
        inner.set_idx_in_xdg_entries_vector(idx);
    } 

    pub fn get_desktop_idx(&self) -> usize {
        let inner = self.get();
        inner.idx_in_xdg_entries_vector
    }

    pub fn set_idx_in_search_result_vector(&mut self, idx: usize) {
        let mut inner = self.get_mut();
        inner.set_idx_in_search_result_vector(idx);
    }

    pub fn get_idx_in_search_result_vector(&self) -> usize {
        let inner = self.get();
        inner.idx_in_search_result_vector
    }
}