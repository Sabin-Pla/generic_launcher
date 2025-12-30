mod badge;
mod day_box;
mod day_marking;
mod user_calendar_data;

use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::{Cast, WidgetExt};
use gtk::subclass::prelude::*;

use chrono::{Month, Datelike};
use gtk::prelude::{BoxExt, GestureSingleExt, GridExt, PopoverExt, TextViewExt};
use gtk4_layer_shell::LayerShell;

use badge::Badge;
use day_box::DayBox;
use day_marking::DayMarking;
pub use day_marking::{MarkingType};
use user_calendar_data::{UserCalendarData, NoteDate};
use super::CenteredWidget;

mod inner {
    use super::*;

    pub struct Calendar {
        pub popover: gtk::Popover,
        pub grid: gtk::Grid,
        pub selected_day: RefCell<Option<(i32, i32)>>,
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
                selected_day: None.into(),
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
        let calendar_outer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        calendar_outer.add_css_class("calendar-outer");
        let calendar_inner = gtk::Box::new(gtk::Orientation::Vertical, 0);
        calendar_inner.append(&calendar.grid);
        calendar_inner.add_css_class("calendar-inner");
        calendar_outer.append(&calendar_inner);


        let notes_entry_scrolled = gtk::ScrolledWindow::new();
        let notes_entry = gtk::TextView::new();
        notes_entry_scrolled.set_child(Some(&notes_entry));
        notes_entry.set_wrap_mode(gtk::WrapMode::Word);
        notes_entry_scrolled.add_css_class("day-notes-window");
        notes_entry.set_hexpand(true);
        calendar_outer.append(&notes_entry_scrolled);
        calendar.popover.set_has_arrow(true);
        calendar.popover.set_child(Some(&calendar_outer));
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
                day_box.add_css_class("selected-day");
                calendar.selected_day.replace(Some((col, row)));
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
            focus_on_hide: &impl gdk::prelude::IsA<gtk::Widget>,
            icon_theme: &gtk::IconTheme) {

        let calendar = &inner::Calendar::from_obj(self);

        let obj = self.clone();
        let obj_left_click = self.clone();

        let popover_left_click = calendar.popover.clone();
        let popover_right_click = calendar.popover.clone();

        let get_clicked_day_box = |popover: &gtk::Popover, x: f64, y: f64| -> Option<DayBox> {
            let mut w = popover.pick(x, y, gtk::PickFlags::DEFAULT);
            while let Some(widget) = w {
                if widget.has_css_class("calendar-daybox") {
                    let day_box = widget.downcast::<DayBox>().expect("non-daybox has css class calendar-daybox");
                    return Some(day_box);
                } else if widget.has_css_class("calendar") {
                    return None;
                }
                w = widget.parent();
            }
            unreachable!("calendar popup does not have calendar css class");
        };

        let outer_box = popover_left_click.child().expect("calendar popover cotnains no box");
                assert!(outer_box.has_css_class("calendar-outer"));
        let text_view = outer_box.last_child().expect("outer_box has no children")
            .first_child()
            .expect("outerbox child (ScrolledWindow) has no child")
            .downcast::<gtk::TextView>()
            .expect("calendar-outer last child child is not text_view");

        apply_note_icon(icon_theme, &text_view);

        let calendar_left_click_handler = move |_gc: &gtk::GestureClick, _: i32, x: f64, y: f64| {

            if let Some(day_box) = get_clicked_day_box(&popover_left_click, x, y) {
                let calendar = inner::Calendar::from_obj(&obj_left_click);
                let (col, row, _, _) = calendar.grid.query_child(&day_box);
                println!("selected day_box: {:?}", (col, row));
                if let Some(last) = calendar.selected_day.replace(Some((col, row))) {
                    let last_selected_day = calendar.grid
                        .child_at(last.0, last.1).expect(
                            &format!("last calendar daybox invalid (col, row) {:?}", last));
                    day_box.add_css_class("selected-day");
                    last_selected_day.remove_css_class("selected-day");
                    if last == (col, row) {
                        calendar.selected_day.replace(None);
                        text_view.set_visible(false);
                    } else {
                        text_view.set_visible(true);
                    }
                    return
                }
                day_box.add_css_class("selected-day");
                text_view.set_visible(true);
            }
        };
        let gesture_left_click = gtk::GestureClick::new();
        gesture_left_click.set_button(1);
        gesture_left_click.connect_pressed(calendar_left_click_handler);
        calendar.popover.add_controller(gesture_left_click);

        let calendar_right_click_handler = move |_gc: &gtk::GestureClick, _: i32, x: f64, y: f64| {
            if let Some(day_box) = get_clicked_day_box(&popover_right_click, x, y) {
                day_box.toggle_marking();
            }
        };
        let gesture_right_click = gtk::GestureClick::new();
        gesture_right_click.set_button(3);
        gesture_right_click.connect_pressed(calendar_right_click_handler);
        calendar.popover.add_controller(gesture_right_click);

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

fn apply_note_icon(icon_theme: &gtk::IconTheme, text_view: &gtk::TextView) {
    let note_taking_icon = icon_theme.lookup_icon(
        "note-taking-symbolic",
        &[],
        32,
        1,
        gtk::TextDirection::None,
        gtk::IconLookupFlags::PRELOAD,
    );
    let note_taking_icon = gtk::Image::from_paintable(Some(&note_taking_icon));
    note_taking_icon.set_icon_size(gtk::IconSize::Large);
    let overlay_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    overlay_box.append(&note_taking_icon);
    overlay_box.add_css_class("note-taking-icon-overlay");
    let icon_label = gtk::Label::builder()
        .label("Type to enter selected day's notes")
        .wrap(true)
        .wrap_mode(gtk::pango::WrapMode::Word)
        .build();
    overlay_box.append(&icon_label);
    let note_taking_icon = CenteredWidget::new(&overlay_box);
    text_view.add_overlay(&note_taking_icon, 0, 0);
}