use std::cell::RefCell;

use gtk::prelude::WidgetExt;
use gtk::subclass::prelude::*;

use gtk::prelude::BoxExt;

use crate::gobject::calendar::{Badge, DayMarking, MarkingType, MarkingCycle};

mod inner {
    use super::*;

    pub struct DayBox {
        pub label: gtk::Label,
        pub marking: RefCell<(super::DayMarking, MarkingCycle)>
    }

    #[gtk::glib::object_subclass]
    impl ObjectSubclass for DayBox {
        const NAME: &'static str = "DayBox";
        type Type = super::DayBox;
        type ParentType = gtk::Widget;

        fn new() -> Self {
            let mut marking_cycle = MarkingType::cycle();
            marking_cycle.next();
            Self {
                label: Default::default(),
                marking: (super::DayMarking::new(), marking_cycle).into()
            }
        }
    }

    impl ObjectImpl for DayBox {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().set_layout_manager(Some(gtk::BinLayout::new()));
        }
    }

    impl WidgetImpl for DayBox {}
}

glib::wrapper! {
    pub struct DayBox(ObjectSubclass<inner::DayBox>)
    @extends gtk::Widget, gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl DayBox {
    pub fn new() -> Self {
        let obj =  glib::Object::new::<Self>();
        obj.add_css_class("calendar-daybox");
        let daybox_inner = gtk::Box::new(gtk::Orientation::Vertical, 0);
        daybox_inner.add_css_class("daybox-inner");
        let day_box = inner::DayBox::from_obj(&obj);
        daybox_inner.append(&day_box.label);

        let overlay = gtk::Overlay::new();
        let badge = Badge::new();
        overlay.add_overlay(&badge);
        overlay.set_parent(&obj);
        daybox_inner.append(&day_box.marking.borrow().0);
        daybox_inner.set_parent(&obj);
        obj
    }

    pub fn set_date(&self, year_number: u32, month_number: u32, day_number: u32) {
        
        // don't forget to zero pad this shit
        println!("{year_number}-{month_number}-{day_number}");

        let day_box = inner::DayBox::from_obj(&self);
        day_box.label.set_text(&day_number.to_string());
    }

    pub fn toggle_marker(&self) {
        let day_box = inner::DayBox::from_obj(&self);
        let mut marking = day_box.marking.borrow_mut();
        marking.1.next().unwrap().set_css(&marking.0);
    }

}
