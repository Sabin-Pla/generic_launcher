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

 #[derive(Clone, Copy, Default)]
pub enum MarkingType {
    #[default] None,
    One,
    Two,
}

impl From<&str> for MarkingType {
    fn from(string: &str) -> Self {
        match string {
            "One" => Self::One,
            "Two" => Self::Two,
            "None"|"" => Self::None,
            _ => panic!("bad marking type string: {string}")
        }
    }
}


impl std::fmt::Display for MarkingType  {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let s = match self {
            Self::One => "One",
            Self::Two => "Two",
            Self::None => "None"
        };
        write!(f, "{}", s)
    }
}

impl MarkingType {
    pub fn set_css(&self, day_marking: &DayMarking) {
        day_marking.remove_css_class("marking-color1");
        day_marking.remove_css_class("marking-color2");
        match self {
            Self::None => (),
            Self::One => day_marking.add_css_class("marking-color1"),
            Self::Two => day_marking.add_css_class("marking-color2")
        }
    }

    pub fn next(&mut self) {
        match self {
            Self::None => *self = Self::One,
            Self::One => *self = Self::Two,
            Self::Two => *self = Self::None
        }
    }
}