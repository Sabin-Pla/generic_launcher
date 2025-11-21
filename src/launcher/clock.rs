use gtk::prelude::*;

use crate::{Duration, HashMap, Rc};

use super::LAUNCHER;

pub fn get_time_str() -> String {
    let date_time  =  chrono::offset::Local::now();
    format!("{}", date_time.format("%a %d/%B %Y %H:%M:%S"))
} 

pub fn set_clock_time(time: &String, clock: &gtk::Label) {
    clock.set_text(&time);
}

fn show_clock() {
    let clock = unsafe {
        LAUNCHER.clock.clone().expect("Clock not initialized")
    };
    let clock_sizes = unsafe { &mut LAUNCHER.clock_sizes };
    let current_monitor = unsafe { LAUNCHER.current_monitor.expect("current monitor not set") }; 
    let clock_sizes = clock_sizes.as_mut();
    let clock_sizes = clock_sizes.expect("clock_sizes not defined");
    let clock = clock.borrow();
    let padded = calculate_clock_padding(&clock);
    let clock_width = clock_sizes.entry(current_monitor).or_insert(padded);
    if *clock_width == 0 {
        *clock_width = padded;
    }
    println!("Showing clock and setting width to {padded}");
    println!("{:?}", &clock_sizes);
    clock.set_opacity(1.0);
    clock.set_size_request(padded, 0);
}


pub fn set_clock_size(
        application_window: &gtk::ApplicationWindow, 
        clock_sizes: &mut Option<HashMap<(i32, i32), i32>>,
        clock: Rc<std::cell::RefCell<gtk::Label>>) {
    let clock = clock.borrow();
    let display = clock.display().monitor_at_surface(
        &application_window.surface().unwrap());
    let rect =  display.unwrap().geometry();
    let (w, h) = (rect.width(), rect.height());
    // width might change substantially when app is opened on different monitor
    // so save a different padded clock width for each monitor and use that
    let clock_sizes = clock_sizes.as_mut();
    let clock_sizes = clock_sizes.expect("clock_sizes not defined");
    let uninitialized = clock_sizes.keys().len() == 0;

    let pad_clock = || {
        if uninitialized || clock.width() == 0 {
            clock.set_visible(false);
            clock.set_size_request(0, 0);
            clock.set_visible(true);
            println!("clock has 0 width {}", clock.width());
            // width may be zero in the event that widget hasn't loaded yet.
            // hide the clock while it loads.
            clock.set_opacity(0.0);
            glib::timeout_add_local_once(Duration::from_millis(20), show_clock);
        } 
        calculate_clock_padding(&clock)
    };
    let provider = gtk::CssProvider::new();
    // todo: make this add half the added width to padding-left. for now we'll just use 20px cause it looks good enough
    provider.load_from_string(".clock { padding-left: 20px }");
    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + 1);

    let clock_width = clock_sizes.entry((w, h)).or_insert_with(pad_clock);
    clock.set_size_request(*clock_width, 40);
}

fn calculate_clock_padding(clock: &gtk::Label) -> i32 {
    let width = clock.width();
    let num_chars = clock.text().len();
    width + ((width / num_chars as i32) as f32 * 3.5) as i32
}
