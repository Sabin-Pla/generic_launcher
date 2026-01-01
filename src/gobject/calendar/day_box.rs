use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::WidgetExt;
use gtk::subclass::prelude::*;

use gtk::prelude::{Cast, BoxExt};

use crate::gobject::calendar::{Badge, Calendar, DayMarking, MarkingType, NoteDate, UserCalendarData};

mod inner {
    use super::*;

    pub struct DayBox {
        pub label: gtk::Label,
        pub date: RefCell<NoteDate>,
        pub marking: RefCell<(super::DayMarking, MarkingType)>,
        pub badge: Badge
    }

    #[gtk::glib::object_subclass]
    impl ObjectSubclass for DayBox {
        const NAME: &'static str = "DayBox";
        type Type = super::DayBox;
        type ParentType = gtk::Widget;

        fn new() -> Self {
            Self {
                label: Default::default(),
                date: NoteDate::from((0, 0, 0)).into(),
                marking: (super::DayMarking::new(),  MarkingType::None).into(),
                badge: Badge::new()
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
        let day_box = inner::DayBox::from_obj(&obj);
        obj.add_css_class("calendar-daybox");
        let daybox_inner = gtk::Box::new(gtk::Orientation::Vertical, 0);
        daybox_inner.add_css_class("daybox-inner");
        let day_box = inner::DayBox::from_obj(&obj);
        daybox_inner.append(&day_box.label);

        let overlay = gtk::Overlay::new();
        overlay.add_overlay(&day_box.badge);
        overlay.set_parent(&obj);
        daybox_inner.append(&day_box.marking.borrow().0);
        daybox_inner.set_parent(&obj);
        obj
    }

    pub fn set_date(&self, year_number: u32, month_number: u32, day_number: u32) {
        self.remove_css_class("today");
        self.remove_css_class("selected-day");
        self.remove_css_class("other-month-daybox");
        let day_box = inner::DayBox::from_obj(&self); 
        day_box.date.replace((year_number, month_number, day_number).into());
        day_box.label.set_text(&day_number.to_string());
        let user_calendar_data = self.get_calendar_data();
        let mut user_calendar_data = user_calendar_data.borrow_mut();

        let note_date = NoteDate::from(*day_box.date.borrow());
        if let Some(note_data) = user_calendar_data.get_date_entry(note_date) {
            Self::set_marking(day_box, &note_data.marking);
            if !note_data.note.is_empty() {
                day_box.badge.set_visible(true);
            } else {
                day_box.badge.set_visible(false);
            }
        } else {
            Self::set_marking(day_box, &MarkingType::default());
            day_box.badge.set_visible(false);
        }
    }

    fn get_calendar_data(&self) -> Rc<RefCell<UserCalendarData>> {
        let mut w = self.parent();
        while let Some(widget) = w {
            if widget.has_css_class("calendar") {
                let calendar = widget.downcast_ref::<Calendar>().expect("non-calendar has css class calendar");
                return calendar.user_calendar_data()
            }
            w = widget.parent();
        }
        panic!("Could not find calendar containing daybox");
    }

    pub fn toggle_marking(&self) {
        let day_box = inner::DayBox::from_obj(&self);
        let mut marking = day_box.marking.borrow_mut();
        marking.1.next();
        marking.1.set_css(&marking.0);
        let note_date = NoteDate::from(*day_box.date.borrow());
        let user_calendar_data = self.get_calendar_data();
        let mut user_calendar_data = user_calendar_data.borrow_mut();
        let note_data = user_calendar_data.get_or_insert_date_entry(note_date);
        note_data.marking = marking.1;
        user_calendar_data.write_contents();
        
    }

    fn set_marking(day_box: &inner::DayBox, marking_type: &MarkingType) {
        let mut marking = day_box.marking.borrow_mut();
        marking.1 = *marking_type;
        marking.1.set_css(&marking.0);
    }

    pub fn get_inner(&self) -> &inner::DayBox {
        inner::DayBox::from_obj(&self)
    }

    pub fn set_badge_visible(&self, state: bool) {
        self.get_inner().badge.set_visible(state)
    }
}
