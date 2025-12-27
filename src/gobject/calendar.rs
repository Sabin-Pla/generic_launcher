use std::cell::RefCell;
use std::rc::Rc;

use gtk::glib::{Object};
use gtk::prelude::{Cast, LayoutManagerExt, WidgetExt};
use gtk::subclass::prelude::*;

use chrono::{Month, Datelike};
use gtk::prelude::{BoxExt, GridExt, PopoverExt};

mod inner {
    use super::*;

    #[derive(Default)]
    pub struct Calendar {
        pub popover: gtk::Popover,
        pub grid: gtk::Grid,
    }

    #[gtk::glib::object_subclass]
    impl ObjectSubclass for Calendar {
        const NAME: &'static str = "Calendar";
        type Type = super::Calendar;
        type ParentType = gtk::Widget;

        fn new() -> Self {
            Self {
                ..Default::default()
            }
        }
    }

    impl ObjectImpl for Calendar {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().set_layout_manager(Some(gtk::BinLayout::new()));
        }
    }

    impl WidgetImpl for Calendar {}

    pub struct DayBox {}

    #[gtk::glib::object_subclass]
    impl ObjectSubclass for DayBox {
        const NAME: &'static str = "DayBox";
        type Type = super::DayBox;
        type ParentType = gtk::Widget;

        fn new() -> Self {
            Self {}
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
    pub struct Calendar(ObjectSubclass<inner::Calendar>)
    @extends gtk::Widget, gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

glib::wrapper! {
    pub struct DayBox(ObjectSubclass<inner::DayBox>)
    @extends gtk::Widget, gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl DayBox {
    pub fn new(day_number: u32) -> Self {
        let obj =  glib::Object::new::<Self>();
        obj.add_css_class("calendar-daybox");
        let day_label = gtk::Label::new(Some(&day_number.to_string()));
        day_label.set_parent(&obj);
        obj
    }
}

impl Calendar {
    pub fn new(
        //    application_window: &gtk::ApplicationWindow,
        //    focus_on_hide: &impl gdk::prelude::IsA<gtk::Widget>
        ) -> Self {

        let obj = glib::Object::new::<Self>();
        let calendar = inner::Calendar::from_obj(&obj);
        calendar.popover.set_parent(&obj);
        /*
        let application_window_connect_show = application_window.clone();
        popover.connect_show(move |_: &gtk::Popover| {
            // must set this to have pointer events fire
            application_window_connect_show.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::OnDemand);
        });

        let application_window_connect_hide = application_window.clone();
        let focus_on_hide = focus_on_hide.clone();
        popover.connect_hide(move |_: &gtk::Popover| {
            application_window_connect_hide.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive);
            focus_on_hide.grab_focus();
        });
        */

        // let day_box = DayBox::new();
        //calendar.grid.attach(&day_box, 0, 0, 1, 1);
        let calendar_inner = gtk::Box::new(gtk::Orientation::Vertical, 0);
        calendar_inner.append(&calendar.grid);
        calendar_inner.add_css_class("calendar-inner");
        calendar.popover.set_child(Some(&calendar_inner));
        calendar.popover.set_position(gtk::PositionType::Top);
        obj
    }

    pub fn update(&self) {
        let calendar = inner::Calendar::from_obj(&self);
    	let now = chrono::offset::Local::now();
    	let month = Month::try_from(now.month() as u8)
    		.expect(&format!("Could not get month of {:?}", &now));
        let prev_month = month.pred();
    	let current_day = now.day();
    	let current_weekday =  now.weekday();
        let first_weekday = now.with_day(1).expect("could not get first day of month").weekday();
        let last_day_of_month = month.num_days(now.year()).expect("failure computing number of days in month");
        let last_month_days = match month {
            Month::January => prev_month.num_days(now.year() - 1)
                .expect("failure computing number of days in previous month (dec)"),
            _ => prev_month.num_days(now.year())
                .expect("failure computing number of days in previous month"),
        };

        // the number of days the first day of the month is from the first sunday
        let first_days_from_sunday = first_weekday.num_days_from_sunday();

        create_weekday_boxes(&calendar.grid);
        let mut row = 1;
        let mut col = 0;
        for i in 0..first_days_from_sunday {
            // add last month's days to calendar
            let day_number = last_month_days as u32 - i;
            let day_box = DayBox::new(day_number);
            day_box.add_css_class("other-month-daybox");
            col = (first_days_from_sunday - (i+1)) as i32;
            calendar.grid.attach(&day_box, col, row, 1, 1);
        }

        for i in 1..last_day_of_month+1 {
            let day_box = DayBox::new(i as u32);
            if i as u32 == current_day {
                day_box.add_css_class("today");
            }
            col += 1;
            if col == 7 {
                col = 0;
                row += 1;
            }
            calendar.grid.attach(&day_box, col, row, 1, 1);
        }

        for (next_month_day, i) in (col..7).enumerate() {
            let day_box = DayBox::new(next_month_day as u32 + 1);
            day_box.add_css_class("other-month-daybox");
            calendar.grid.attach(&day_box, i, row, 1, 1);
        }
    }

    pub fn open(&self) {
        let calendar = &inner::Calendar::from_obj(self);
        self.update();
        calendar.popover.popup();
    }
}

fn create_weekday_boxes(grid: &gtk::Grid) {
    let day_labels_en = ["S", "M", "T", "W", "T", "F", "S"];
    for (i, weekday) in day_labels_en.iter().enumerate() {
        let label = gtk::Label::new(Some(weekday));
        label.add_css_class("weekday-label");
        grid.attach(&label, i as i32, 0, 1, 1);
    }
}


