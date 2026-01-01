mod badge;
mod day_box;
mod day_marking;
mod user_calendar_data;

use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::{Cast, WidgetExt};
use gtk::subclass::prelude::*;

use chrono::{Month, Datelike};
use gtk::prelude::{BoxExt, ObjectExt, TextBufferExt, GestureSingleExt, GridExt, PopoverExt, TextViewExt};
use gtk4_layer_shell::LayerShell;

use badge::Badge;
use day_box::DayBox;
use day_marking::DayMarking;
pub use day_marking::{MarkingType};
use user_calendar_data::{UserCalendarData, NoteDate};
use super::CenteredWidget;

const NOTE_OVERLAY_EDITABLE:    &str = "Type to enter selected day's notes";
const NOTE_OVERLAY_NONEDITABLE: &str = "No note was entered for selected day";

mod inner {
    use super::*;

    pub struct Calendar {
        pub popover: gtk::Popover,
        pub grid: gtk::Grid,
        pub selected_date: RefCell<(chrono::NaiveDate, gtk::Label)>,
        pub selected_day: RefCell<Option<(i32, i32)>>,
        pub note_overlay_box: RefCell<gtk::Box>,
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
                selected_date: (chrono::Local::now().date_naive(), gtk::Label::new(None)).into(),
                note_overlay_box: gtk::Box::new(gtk::Orientation::Horizontal, 0).into(),
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
        calendar_inner.append(&create_month_selector(&calendar));
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

    fn load_month(&self) {
        let calendar = inner::Calendar::from_obj(&self);
    	let selected_date = calendar.selected_date.borrow().clone();
        selected_date.1.set_text(
            &format!("{}", selected_date.0.format_localized("%B %Y", chrono::Locale::default())));
        let selected_date = selected_date.0;
    	let month = Month::try_from(selected_date.month() as u8)
    		.expect(&format!("Could not get month of {:?}", &selected_date));
        let prev_month = month.pred();
    	let current_day = selected_date.day();
        let first_weekday = selected_date.with_day(1).expect("could not get first day of month").weekday();
        let last_day_of_month = month.num_days(selected_date.year()).expect("failure computing number of days in month");
        let mut last_month_year = selected_date.year();
        let next_month = month.succ();
        let next_month_year = match next_month {
            Month::January => selected_date.year() + 1,
            _ => selected_date.year()
        };
        let last_month_days = match month {
            Month::January => {
                last_month_year = selected_date.year() - 1;
                prev_month.num_days(last_month_year)
                .expect("failure computing number of days in previous month (dec)")
            },
            _ => prev_month.num_days(selected_date.year())
                .expect("failure computing number of days in previous month"),
        };

        calendar.selected_day.replace(None);

        // the number of days the first day of the month is from the first sunday
        let first_days_from_sunday = first_weekday.num_days_from_sunday();

        let mut row = 1;
        let mut col = 0;
        for i in 0..first_days_from_sunday {
            // add last month's days to calendar
            let day_number = last_month_days as u32 - (first_days_from_sunday - i - 1);
            let day_box = get_day_from_calendar_grid(&calendar.grid, (col, row));
            day_box.set_date(last_month_year as u32, prev_month.number_from_month(), day_number);
            day_box.add_css_class("other-month-daybox");
            col += 1;
        }

        let note_overlay_box = calendar.note_overlay_box.borrow();
        let now = chrono::Local::now().date_naive();
        let note_entry_text_view = get_overlay_box_textview(&note_overlay_box);
        note_overlay_box.set_visible(false);
        note_entry_text_view.set_visible(false);
        for i in 1..last_day_of_month+1 {
            let day_box =  get_day_from_calendar_grid(&calendar.grid, (col, row));
            day_box.set_date(selected_date.year() as u32, month.number_from_month(), i as u32);
            if i as u32 == current_day && month.number_from_month() == now.month() && selected_date.year() == now.year() {
                day_box.add_css_class("today");
                day_box.add_css_class("selected-day");
                calendar.selected_day.replace(Some((col, row)));
                note_overlay_box.set_visible(true);
                note_entry_text_view.set_visible(true);
                display_day_notes(&calendar, &day_box, &note_entry_text_view);
            }
            col += 1;
            if col == 7 {
                col = 0;
                row += 1;
            }
        }

        let mut next_month_day = 1;
        while row < 7 {
            let day_box = get_day_from_calendar_grid(&calendar.grid, (col, row));
            day_box.set_date(next_month_year as u32, next_month.number_from_month(), next_month_day);
            day_box.add_css_class("other-month-daybox");
            col += 1;
            next_month_day += 1;
            if col == 7 {
                col = 0;
                row += 1;
            }
        }
    }

    pub fn open(&self) {
        let calendar = self.get_inner();
        calendar.selected_date.borrow_mut().0 = chrono::Local::now().date_naive();
        self.load_month();
        calendar.popover.popup();
    }

    pub fn initialize(&self,
            application_window: &gtk::ApplicationWindow, 
            focus_on_hide: &impl gdk::prelude::IsA<gtk::Widget>,
            icon_theme: &gtk::IconTheme) {

        let calendar = self.get_inner();

        let outer_box = calendar.popover.child().expect("calendar popover cotnains no box");
                assert!(outer_box.has_css_class("calendar-outer"));
        let note_text_view = outer_box.last_child().expect("outer_box has no children")
            .first_child()
            .expect("outerbox child (ScrolledWindow) has no child")
            .downcast::<gtk::TextView>()
            .expect("calendar-outer last child child is not TextView");

        apply_note_icon(self, icon_theme, &note_text_view);
        let gesture_left_click = gtk::GestureClick::new();
        gesture_left_click.set_button(1);
        gesture_left_click.connect_pressed(
            left_click_handler(calendar.popover.clone(), self.clone(), note_text_view.clone()));
        calendar.popover.add_controller(gesture_left_click);

        let popover_right_click = calendar.popover.clone();
        let calendar_right_click_handler = move |_gc: &gtk::GestureClick, _: i32, x: f64, y: f64| {
            if let Some(day_box) = get_clicked_day_box(&popover_right_click, x, y) {
                day_box.toggle_marking();
            }
        };

        let gesture_right_click = gtk::GestureClick::new();
        gesture_right_click.set_button(3);
        gesture_right_click.connect_pressed(calendar_right_click_handler);
        calendar.popover.add_controller(gesture_right_click);

        let note_text_view_connect_show = note_text_view.clone();
        let application_window_connect_show = application_window.clone();
        let obj_connect_show = self.clone();

        calendar.popover.connect_show(move |_: &gtk::Popover| {
            // must set this to have pointer events fire
            let calendar = obj_connect_show.get_inner();
            display_selected_day_notes(calendar, &note_text_view_connect_show);
            let bounds = obj_connect_show
                .compute_bounds(&application_window_connect_show)
                .expect("could not compute bounds of volume popover");
            calendar.popover.set_offset(0, -bounds.y() as i32);
            application_window_connect_show.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::OnDemand);
        });

        let application_window_connect_hide = application_window.clone();
        let focus_on_hide = focus_on_hide.clone();
        let obj_connect_hide = self.clone();

        calendar.popover.connect_hide(move |_: &gtk::Popover| {
            let calendar = obj_connect_hide.get_inner();
            application_window_connect_hide.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive);
            resync_selected_note_changes(calendar, &note_text_view);
            if let Some(selected_day) = get_selected_day(&calendar) {
                selected_day.remove_css_class("selected-day");
            }
            focus_on_hide.grab_focus();
        });

        attach_weekday_boxes(&calendar.grid);
        attach_day_boxes(&calendar.grid);
    }

    pub fn user_calendar_data(&self) -> Rc<RefCell<UserCalendarData>> {
        let calendar = &inner::Calendar::from_obj(self);
        calendar.user_calendar_data.clone()
    }

    pub fn get_inner(&self) -> &inner::Calendar {
        inner::Calendar::from_obj(&self)
    }
}

fn get_selected_day(calendar: &inner::Calendar) -> Option<DayBox> {
    let selected_day = calendar.selected_day.borrow();
    selected_day.map(|day| get_day_from_calendar_grid(&calendar.grid, day)) 
}

fn display_selected_day_notes(calendar: &inner::Calendar, note_entry_text_view: &gtk::TextView) {
    if let Some(selected_day) = get_selected_day(calendar) {
        display_day_notes(calendar, &selected_day, note_entry_text_view);
    }
}

fn display_day_notes(calendar: &inner::Calendar, selected_day: &DayBox, note_entry_text_view: &gtk::TextView) {
    let user_calendar_data = calendar.user_calendar_data.borrow_mut();
    let selected_day = selected_day.get_inner();
    let overlay_box = calendar.note_overlay_box.borrow();
    let date = *selected_day.date.borrow();
    match before_yesterday_12_am(date) {
        true => set_overlay_box_message(&overlay_box, note_entry_text_view, false),
        false => set_overlay_box_message(&overlay_box, note_entry_text_view, true)
    }
    if let Some(date_note) = user_calendar_data.get_date_notes(date) {
        note_entry_text_view.buffer().set_text(&date_note);
        if date_note.trim().is_empty() {
            overlay_box.set_visible(true);
            note_entry_text_view.grab_focus();
        } else {
            overlay_box.set_visible(false);
        }
    } else {
        note_entry_text_view.buffer().set_text("");
        overlay_box.set_visible(true);
        note_entry_text_view.grab_focus();
    }
}

fn set_overlay_box_message(overlay_box: &gtk::Box, note_entry_text_view: &gtk::TextView, editable: bool) {
    let note_icon = overlay_box.first_child().expect("overlay box has no child");
    let label = note_icon.next_sibling()
        .expect("note icon has no label sibling")
        .downcast::<gtk::Label>()
        .expect("note icon sibling is not label");
    if editable {
        note_icon.set_visible(true);
        note_entry_text_view.set_editable(true);
        label.set_text(NOTE_OVERLAY_EDITABLE);
    } else {
        note_icon.set_visible(false);
        note_entry_text_view.set_editable(false);
        label.set_text(NOTE_OVERLAY_NONEDITABLE);
    }
}

fn before_yesterday_12_am(date: NoteDate) -> bool {
    let today = chrono::Local::now().date_naive();
    let yesterday = today.pred(); // calendar-correct

    let given_date = chrono::NaiveDate::from_ymd_opt(
        date.year as i32,
        date.month,
        date.day,
    ).unwrap();

    given_date < yesterday
}

fn left_click_handler(
        popover: gtk::Popover, 
        calendar: Calendar,
        note_entry_text_view: gtk::TextView
    ) -> impl Fn(&gtk::GestureClick, i32, f64, f64) {
    move |_gc: &gtk::GestureClick, _: i32, x: f64, y: f64| {
        let calendar = calendar.get_inner();
        if let Some(day_box) = get_clicked_day_box(&popover, x, y) {
            let (col, row, _, _) = calendar.grid.query_child(&day_box);
            println!("selected day_box: {:?}", (col, row));
            if let Some(last) = calendar.selected_day.replace(Some((col, row))) {
                let last_selected_day = get_day_from_calendar_grid(&calendar.grid, last);
                day_box.add_css_class("selected-day");
                last_selected_day.remove_css_class("selected-day");
                resync_note_changes(calendar, &note_entry_text_view, &last_selected_day);
                if last == (col, row) {
                    // user just reselected the same date.
                    calendar.selected_day.replace(None);
                    note_entry_text_view.set_visible(false);
                    return;
                }
                note_entry_text_view.set_visible(true);
                display_day_notes(calendar, &day_box, &note_entry_text_view);
                
            } else {
                display_selected_day_notes(calendar, &note_entry_text_view);
                day_box.add_css_class("selected-day");
                note_entry_text_view.set_visible(true);
            }
        } else if let Some(is_next) = get_clicked_month_selector(&popover, x, y) {
            resync_selected_note_changes(calendar, &note_entry_text_view);
            let mut selected_date = calendar.selected_date.borrow_mut();
            let one_month = chrono::Months::new(1);
            let new_date = match is_next {
                true => selected_date.0 + one_month,
                false => selected_date.0 - one_month
            };
            selected_date.0 = new_date;
            drop(selected_date);
            calendar.obj().load_month();
        }
    }
}

fn get_clicked_month_selector(popover: &gtk::Popover, x: f64, y: f64) -> Option<bool> {
    let mut w = popover.pick(x, y, gtk::PickFlags::DEFAULT);
    while let Some(widget) = w {
        if widget.has_css_class("month-selector-last") {
            return Some(false);
        } else if widget.has_css_class("month-selector-next") {
            return Some(true);
        } else if widget.has_css_class("calendar") {
            return None;
        }
        w = widget.parent();
    }
    unreachable!("calendar popup does not have calendar css class");
}

fn get_clicked_day_box(popover: &gtk::Popover, x: f64, y: f64) -> Option<DayBox> {
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
}

fn resync_selected_note_changes(calendar: &inner::Calendar, text_view: &gtk::TextView) {
    if let Some(selected_day) = get_selected_day(calendar) {
        resync_note_changes(calendar, text_view, &selected_day);
    }
}

fn resync_note_changes(calendar: &inner::Calendar, note_entry_text_view: &gtk::TextView, day_box: &DayBox) {
    let mut user_calendar_data = calendar.user_calendar_data.borrow_mut();
    let buffer = note_entry_text_view.buffer();
    let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
    user_calendar_data.resync_note_changes(
        *day_box.get_inner().date.borrow(), text.to_string());
}

fn get_day_from_calendar_grid(grid: &gtk::Grid, selected_day: (i32, i32)) -> DayBox {
    grid.child_at(selected_day.0, selected_day.1)
        .expect(&format!("last calendar daybox invalid (col, row) {:?}", selected_day))
        .downcast::<DayBox>()
        .expect("Calendar contains non-DayBox widget")
}

fn attach_day_boxes(grid: &gtk::Grid) {
    let mut row = 1;
    let mut col = 0;
    for _ in 0..42 {
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

fn create_month_selector(calendar: &inner::Calendar) -> gtk::CenterBox {
    let month_selector = gtk::CenterBox::builder()
            .orientation(gtk::Orientation::Horizontal)
            .build();
    month_selector.add_css_class("month-selector");
    let last_month_button = gtk::Label::new(Some("<"));
    last_month_button.add_css_class("month-selector-last");
    let next_month_button = gtk::Label::new(Some(">"));
    next_month_button.add_css_class("month-selector-next");

    month_selector.set_center_widget(Some(&calendar.selected_date.borrow().1));
    month_selector.set_start_widget(Some(&last_month_button));
    month_selector.set_end_widget(Some(&next_month_button));
    month_selector
}

fn apply_note_icon(calendar_obj: &Calendar, icon_theme: &gtk::IconTheme, note_text_view: &gtk::TextView) {
    let calendar = calendar_obj.get_inner();
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
    let overlay_box = calendar.note_overlay_box.borrow();
    overlay_box.append(&note_taking_icon);
    overlay_box.add_css_class("note-taking-icon-overlay");
    let icon_label = gtk::Label::builder()
        .wrap(true)
        .wrap_mode(gtk::pango::WrapMode::Word)
        .build();
    overlay_box.append(&icon_label);
    let note_taking_icon = CenteredWidget::new(&*overlay_box);
    note_text_view.add_overlay(&note_taking_icon, 0, 0);
    note_text_view.set_cursor_visible(false);
    attach_note_text_view_key_handler(&note_text_view, &overlay_box, &calendar_obj)
}

fn attach_note_text_view_key_handler(note_entry_text_view: &gtk::TextView, overlay_box: &gtk::Box, calendar: &Calendar) {
    let overlay_box = overlay_box.clone();
    let note_entry_text_view = note_entry_text_view.clone();
    let calendar = calendar.clone();
    note_entry_text_view.buffer().connect_notify_local(Some("text"), move |buffer, _| {
        let selected_day = get_selected_day(calendar.get_inner())
            .expect("note_entry_text_view is open but could not find selected day");
        if buffer.text(&buffer.start_iter(), &buffer.end_iter(), false).is_empty() {
            overlay_box.set_visible(true); 
            selected_day.set_badge_visible(false);
            note_entry_text_view.set_cursor_visible(false);
        } else {
            overlay_box.set_visible(false); 
            selected_day.set_badge_visible(true);
            note_entry_text_view.set_cursor_visible(true);
        }
    });
}

fn get_overlay_box_textview(overlay_box: &gtk::Box) -> gtk::TextView {
    overlay_box
        .parent()
        .expect("note overlay box has no parent")
        .parent()
        .expect("note overlay box parent has no parent")
        .parent()
        .expect("note overlay box parent parent has no parent")
        .downcast::<gtk::TextView>()
        .expect("note overlay box parent parent parent is not textview")
}