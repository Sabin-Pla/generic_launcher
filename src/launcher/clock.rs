use gtk::prelude::*;

use crate::{Duration, HashMap, Rc, RefCell};
use std::collections::hash_map::Entry::{Vacant, Occupied};
use crate::launcher::Launcher;


pub fn get_time_str() -> String {
    let date_time  =  chrono::offset::Local::now();
    format!("{}", date_time.format("%a %d/%B %Y %H:%M:%S"))
} 

pub fn set_clock_time(time: &String, clock: &gtk::Label) {
    clock.set_text(&time);
}

fn show_clock(launcher: Rc<RefCell<Launcher>>) {
     println!("qqq");
    let mut launcher = launcher.borrow_mut();
     println!("qqq");
    let current_monitor = launcher.current_monitor.clone().expect("current monitor not set"); 
    let clock = launcher.clock.clone().expect("Clock not initialized");
    let clock_sizes = &mut launcher.clock_sizes;
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
        launcher_cell: Rc<RefCell<Launcher>>) {
    let launcher_cell_clock = launcher_cell.clone();
    let mut launcher = launcher_cell.borrow_mut();

    let clock = launcher.clock.clone().expect("cannot set uninitialized clock");
    let clock = clock.borrow();
    let display = clock.display().monitor_at_surface(
        &application_window.surface().unwrap());
    let rect =  display.unwrap().geometry();
    let (w, h) = (rect.width(), rect.height());
    // width might change substantially when app is opened on different monitor
    // so save a different padded clock width for each monitor and use that
    let clock_sizes = launcher.clock_sizes.as_mut();
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
            let show_clock_wrapper = || {
                show_clock(launcher_cell_clock)
            };
            glib::timeout_add_local_once(Duration::from_millis(20), show_clock_wrapper);
        } 
        calculate_clock_padding(&clock)
    };
    let provider = gtk::CssProvider::new();
    // todo: make this add half the added width to padding-left. for now we'll just use 20px cause it looks good enough
    provider.load_from_string(".clock { padding-left: 20px; }");
    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + 1);

    let clock_width = match clock_sizes.entry((w, h)) {
        Occupied(entry, ..) => {
            let entry = *entry.get();
            drop(launcher);
            entry
        },
        Vacant(..) => {
            drop(launcher);
            println!("v");
            let p = pad_clock();
            println!("v");
            let mut launcher = launcher_cell.borrow_mut();
            let clock_sizes = launcher.clock_sizes.as_mut();
            let clock_sizes = clock_sizes.expect("clock_sizes not defined");
            clock_sizes.insert((w, h) , p);
            p
        }
    };
    let mut launcher = launcher_cell.borrow_mut();
    clock.set_size_request(clock_width, 40);
}

fn calculate_clock_padding(clock: &gtk::Label) -> i32 {
    let width = clock.width();
    let num_chars = clock.text().len();
    width + ((width / num_chars as i32) as f32 * 3.5) as i32
}
