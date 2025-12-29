mod badge;
mod day_box;
mod day_marking;
mod user_calendar_data;

use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::{Cast, WidgetExt};
use gtk::subclass::prelude::*;

use chrono::{Month, Datelike};
use gtk::prelude::{BoxExt, GridExt, PopoverExt};
use gtk4_layer_shell::LayerShell;

use badge::Badge;
use day_box::DayBox;
use day_marking::DayMarking;
pub use day_marking::{MarkingType};
use user_calendar_data::{UserCalendarData, NoteDate};

mod inner {
    use super::*;

    pub struct Calendar {
        pub popover: gtk::Popover,
        pub grid: gtk::Grid,
        pub user_calendar_data: Rc<RefCell<UserCalendarData>>
    }

    #[gtk::glib::object_subclass]
    impl ObjectSubclass for Calendar {
        const NAME: &'static str = "Calendar";
        type Type = super::Calendar;
        type ParentType = gtk::Widget;

        fn new() -> Self {
            let user_calendar_data = UserCalendarData::load();
            Self {
                user_calendar_data: Rc::new(RefCell::new(user_calendar_data.into())),
                grid: gtk::Grid::new(),
                popover: gtk::Popover::new()
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
}

glib::wrapper! {
    pub struct Calendar(ObjectSubclass<inner::Calendar>)
    @extends gtk::Widget, gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl Calendar {
    pub fn new() -> Self {
        let obj = glib::Object::new::<Self>();
        obj.add_css_class("calendar");
        let calendar = inner::Calendar::from_obj(&obj);
        calendar.popover.set_parent(&obj);
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
        let first_weekday = now.with_day(1).expect("could not get first day of month").weekday();
        let last_day_of_month = month.num_days(now.year()).expect("failure computing number of days in month");
        let mut last_month_year = now.year();
        let next_month = month.succ();
        let next_month_year = match next_month {
            Month::January => now.year() + 1,
            _ => now.year()
        };
        let last_month_days = match month {
            Month::January => {
                last_month_year = now.year() - 1;
                prev_month.num_days(last_month_year)
                .expect("failure computing number of days in previous month (dec)")
            },
            _ => prev_month.num_days(now.year())
                .expect("failure computing number of days in previous month"),
        };

        // the number of days the first day of the month is from the first sunday
        let first_days_from_sunday = first_weekday.num_days_from_sunday();

        let get_daybox = |col, row| -> DayBox {
            calendar.grid.child_at(col, row).expect("invalid calendar grid (1)")
                .clone()
                .downcast::<DayBox>()
                .expect("Calendar contains non-DayBox widget")
        };

        let mut row = 1;
        let mut col = 0;
        for i in 0..first_days_from_sunday {
            // add last month's days to calendar
            let day_number = last_month_days as u32 - i;
            col = (first_days_from_sunday - (i+1)) as i32;
            let day_box = get_daybox(col, row);
            day_box.set_date(last_month_year as u32, prev_month.number_from_month(), day_number);
            day_box.add_css_class("other-month-daybox");
        }

        for i in 1..last_day_of_month+1 {
            col += 1;
            if col == 7 {
                col = 0;
                row += 1;
            }
            let day_box =  get_daybox(col, row);
            day_box.set_date(now.year() as u32, month.number_from_month(), i as u32);
            if i as u32 == current_day {
                day_box.add_css_class("today");
            }
        }

        for (next_month_day, i) in (col..7).enumerate() {
            let day_box =  get_daybox(i, row);
            day_box.add_css_class("other-month-daybox");
            day_box.set_date(next_month_year as u32, next_month.number_from_month(), next_month_day as u32 + 1);
        }
    }

    pub fn open(&self) {
        let calendar = &inner::Calendar::from_obj(self);
        self.update();
        calendar.popover.popup();
    }

    pub fn initialize(&self,
            application_window: &gtk::ApplicationWindow, 
            focus_on_hide: &impl gdk::prelude::IsA<gtk::Widget>) {

        let calendar = &inner::Calendar::from_obj(self);

        let obj = self.clone();

        let popover_click = calendar.popover.clone();
        let calendar_click_handler = move |_gc: &gtk::GestureClick, _: i32, x: f64, y: f64| {
            let mut w = popover_click.pick(x, y, gtk::PickFlags::DEFAULT);
            while let Some(widget) = w {
                if widget.has_css_class("calendar-daybox") {
                    let day_box = widget.downcast::<DayBox>().expect("non-daybox has css class calendar-daybox");
                    day_box.toggle_marking();
                    break;
                } else if widget.has_css_class("calendar-inner") {
                    break;
                }
                w = widget.parent();
            }
        };
        let gesture_click = gtk::GestureClick::new();
        gesture_click.connect_pressed(calendar_click_handler);
        calendar.popover.add_controller(gesture_click);

        let application_window_connect_show = application_window.clone();
        calendar.popover.connect_show(move |_: &gtk::Popover| {
            // must set this to have pointer events fire
            let calendar = &inner::Calendar::from_obj(&obj);
            let bounds = obj.compute_bounds(&application_window_connect_show).expect(
                "could not compute bounds of volume popover");
            calendar.popover.set_offset(0, -bounds.y() as i32);
            application_window_connect_show.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::OnDemand);
        });

        let application_window_connect_hide = application_window.clone();
        let focus_on_hide = focus_on_hide.clone();
        calendar.popover.connect_hide(move |_: &gtk::Popover| {
            application_window_connect_hide.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive);
            focus_on_hide.grab_focus();
        });

        attach_weekday_boxes(&calendar.grid);
        create_day_boxes(&calendar.grid);
    }

    pub fn user_calendar_data(&self) -> Rc<RefCell<UserCalendarData>> {
        let calendar = &inner::Calendar::from_obj(self);
        calendar.user_calendar_data.clone()
    }
}

fn create_day_boxes(grid: &gtk::Grid) {
    let mut row = 1;
    let mut col = 0;
    for _ in 0..35 {
        let day_box = DayBox::new();
        grid.attach(&day_box, col, row, 1, 1);
        col += 1;
        if col == 7 {
            col = 0;
            row += 1;
        }
    }
}

fn attach_weekday_boxes(grid: &gtk::Grid) {
    let day_labels_en = ["S", "M", "T", "W", "T", "F", "S"];
    // TODO: ユーザーのPCのロケールが日本語または中国語の場合は以下を使用する
    // let day_labels_zhjp = ["日", "月", "火", "水", "木", "金", "土"];

    for (i, weekday) in day_labels_en.iter().enumerate() {
        let label = gtk::Label::new(Some(weekday));
        label.add_css_class("weekday-label");
        grid.attach(&label, i as i32, 0, 1, 1);
    }
}
