use std::cell::RefCell;
use std::rc::Rc;

use gtk::glib::{Object};
use gtk::prelude::{Cast, LayoutManagerExt, WidgetExt};
use gtk::subclass::prelude::*;

use chrono::{Month, Datelike};
use gtk::prelude::{BoxExt, GridExt, PopoverExt};
use gtk4_layer_shell::LayerShell;

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


    pub struct Badge {}

    #[gtk::glib::object_subclass]
    impl ObjectSubclass for Badge {
        const NAME: &'static str = "Badge";
        type Type = super::Badge;
        type ParentType = gtk::Widget;

        fn new() -> Self {
            Self {}
        }
    }

    impl ObjectImpl for Badge {}

    impl WidgetImpl for Badge {}


}

glib::wrapper! {
    pub struct Calendar(ObjectSubclass<inner::Calendar>)
    @extends gtk::Widget, gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

glib::wrapper! {
    pub struct DayBox(ObjectSubclass<inner::DayBox>)
    @extends gtk::Widget, gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

glib::wrapper! {
    pub struct DayMarking(ObjectSubclass<inner::DayMarking>)
    @extends gtk::Widget, gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

glib::wrapper! {
    pub struct Badge(ObjectSubclass<inner::Badge>)
    @extends gtk::Widget, gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl Badge {
    pub fn new() -> Self {
        let obj =  glib::Object::new::<Self>();
        obj.add_css_class("note-badge");
        obj.set_halign(gtk::Align::End);    
        obj.set_valign(gtk::Align::Start);
        obj
    }
}


impl DayMarking {
    pub fn new() -> Self {
        let obj =  glib::Object::new::<Self>();
        obj.add_css_class("day-marking");
        obj
    }
}

enum MarkingType {
    None,
    One,
    Two,
}

type MarkingCycle = std::iter::Cycle<std::slice::Iter<'static, MarkingType>>;

impl MarkingType {
    fn cycle() -> MarkingCycle {
        [Self::None, Self::One, Self::Two].iter().cycle()
    }

    fn set_css(&self, day_marking: &DayMarking) {
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

    pub fn set_day_text(&self, day_number: u32) {
        let day_box = inner::DayBox::from_obj(&self);
        day_box.label.set_text(&day_number.to_string());
    }

    pub fn toggle_marker(&self) {
        let day_box = inner::DayBox::from_obj(&self);
        let mut marking = day_box.marking.borrow_mut();
        marking.1.next().unwrap().set_css(&marking.0);
    }

}

impl Calendar {
    pub fn new() -> Self {
        let obj = glib::Object::new::<Self>();
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
            day_box.set_day_text(day_number);
            day_box.add_css_class("other-month-daybox");
        }

        for i in 1..last_day_of_month+1 {
            col += 1;
            if col == 7 {
                col = 0;
                row += 1;
            }
            let day_box =  get_daybox(col, row);;
            day_box.set_day_text(i as u32);
            if i as u32 == current_day {
                day_box.add_css_class("today");
            }
        }

        for (next_month_day, i) in (col..7).enumerate() {
            let day_box =  get_daybox(i, row);
            day_box.add_css_class("other-month-daybox");
            day_box.set_day_text(next_month_day as u32 + 1);
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
                    day_box.toggle_marker();
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
}

fn create_day_boxes(grid: &gtk::Grid) {
    let mut row = 1;
    let mut col = 0;
    for i in 0..35 {
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


