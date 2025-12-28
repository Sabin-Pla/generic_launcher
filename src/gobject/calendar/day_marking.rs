use gtk::prelude::{WidgetExt};
use gtk::subclass::prelude::*;

mod inner {
    use super::*;

    pub struct DayMarking {}

    #[gtk::glib::object_subclass]
    impl ObjectSubclass for DayMarking {
        const NAME: &'static str = "DayMarking";
        type Type = super::DayMarking;
        type ParentType = gtk::Widget;

        fn new() -> Self {
            Self {}
        }
    }

    impl ObjectImpl for DayMarking {}
    impl WidgetImpl for DayMarking {}
}

glib::wrapper! {
    pub struct DayMarking(ObjectSubclass<inner::DayMarking>)
    @extends gtk::Widget, gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}


impl DayMarking {
    pub fn new() -> Self {
        let obj =  glib::Object::new::<Self>();
        obj.add_css_class("day-marking");
        obj
    }
}

pub enum MarkingType {
    None,
    One,
    Two,
}

pub type MarkingCycle = std::iter::Cycle<std::slice::Iter<'static, MarkingType>>;

impl MarkingType {
    pub fn cycle() -> MarkingCycle {
        [Self::None, Self::One, Self::Two].iter().cycle()
    }

    pub fn set_css(&self, day_marking: &DayMarking) {
        match self {
            Self::None => day_marking.remove_css_class("marking-color2"),
            Self::One => day_marking.add_css_class("marking-color1"),
            Self::Two => {
                day_marking.add_css_class("marking-color2"); 
                day_marking.remove_css_class("marking-color1");
            }
        }
    }
}