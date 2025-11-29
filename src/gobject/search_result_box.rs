use std::cell::RefMut;
use std::cell::{Ref, RefCell};

use gtk::Accessible;
use gtk::Actionable;
use gtk::Buildable;
use gtk::ConstraintTarget;
use gtk::subclass::prelude::*;

pub struct SearchResultBoxData {
    pub idx_in_container: usize,
    pub idx_in_xdg_entries_vector: usize,
    pub idx_in_search_result_vector: usize,
}

impl SearchResultBoxData {
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
            idx_in_search_result_vector: 0,
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

    pub struct SearchResultBox(pub RefCell<SearchResultBoxData>);

    impl ObjectImpl for SearchResultBox {}
    impl WidgetImpl for SearchResultBox {
        fn grab_focus(&self) -> bool {
            self.parent_grab_focus()
        }
    }
    impl ButtonImpl for SearchResultBox {}

    #[gtk::glib::object_subclass]
    impl ObjectSubclass for SearchResultBox {
        const NAME: &'static str = "SearchResultBox";
        type Type = super::SearchResultBox;
        type ParentType = gtk::Button;

        fn new() -> Self {
            Self(SearchResultBoxData::new().into())
        }
    }
}

glib::wrapper! {
    pub struct SearchResultBox(ObjectSubclass<inner::SearchResultBox>)
    @extends gtk::Widget, gtk::Button, gtk::Frame, ConstraintTarget, Buildable, Accessible, Actionable;
}

impl SearchResultBox {
    pub fn new(box_idx: usize) -> Self {
        let obj = gtk::glib::Object::new::<Self>();
        let result_box_data = SearchResultBoxData::from(box_idx);
        *inner::SearchResultBox::from_obj(&obj).0.borrow_mut() = result_box_data;
        obj
    }

    pub fn get(&self) -> Ref<'_, SearchResultBoxData> {
        inner::SearchResultBox::from_obj(self).0.borrow()
    }

    pub fn get_mut(&self) -> RefMut<'_, SearchResultBoxData> {
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
