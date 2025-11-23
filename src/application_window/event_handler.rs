use crate::launcher::{Launcher, clock};
use crate::{Arc, Mutex};
use crate::application_window::event_handler;
use gtk::prelude::EditableExt;
use gtk::PropagationPhase;
use gtk::prelude::WidgetExt;


pub fn screenshot_click_handler(_gc: &gtk::GestureClick, _: i32, _: f64, _: f64) {
    /*
    unsafe {
        LAUNCHER.select_screenshot_button();
        LAUNCHER.launch_selected_application();
        LAUNCHER.hide_window();
    }*/
}

    pub fn attach_screenshot_handlers(launcher: Arc<Mutex<Launcher>>) {
        let ecm = gtk::EventControllerMotion::builder()
            .propagation_phase(gtk::PropagationPhase::Capture).build();

        let enter_launcher = launcher.clone();
        let screenshot_enter_handler = move |_: &gtk::EventControllerMotion,  _: f64, _: f64| {
            let mut enter_launcher = enter_launcher.lock().unwrap();
            enter_launcher.select_screenshot_button();
        };

        let screenshot_leave_handler = move |_: &gtk::EventControllerMotion| {
            let mut launcher = launcher.lock().unwrap();
            launcher.focus_text_input();
        };

        ecm.connect_enter(screenshot_enter_handler);
        ecm.connect_leave(screenshot_leave_handler);
    }

pub fn attach_window_key_handler(
        application_window: &mut gtk::ApplicationWindow, 
        launcher: Arc<Mutex<Launcher>>) {

    let eck = gtk::EventControllerKey::builder()
        .propagation_phase(PropagationPhase::Capture).build();

    let key_handler = move |
        _: &gtk::EventControllerKey,
        key: gdk::Key,
        _: u32,
        _: gdk::ModifierType| -> gtk::glib::Propagation {

        println!("key {}", key);
        let mut launcher = launcher.lock().unwrap();
        match key {
            gdk::Key::Escape => launcher.hide_window(),
            gdk::Key::Return => launcher.handle_enter_key(),
            gdk::Key::Down => launcher.scroll_search_results_down(),
            _ => ()
        };

        match launcher.selected_search_idx {
            Some(_) => (),
            None => return gtk::glib::Propagation::Proceed,
        };

        match key {
            gdk::Key::BackSpace => {
                launcher.focus_text_input();
                let input = launcher.text_input.clone().unwrap();
                let pos = launcher.search_context.buf.len() as i32;
                input.delete_text(pos -1, pos);
                input.select_region(pos - 1, pos - 1);
                return gtk::glib::Propagation::Proceed;
            },
            _ => ()
        };

        let key_unicode = key.to_unicode();
        match key_unicode {
            Some('\r') => (),
            Some(character) => {
                launcher.focus_text_input();
                let input = launcher.text_input.clone().unwrap();
                let mut pos = launcher.search_context.buf.len() as i32;
                input.insert_text(
                   &character.to_string(), 
                   &mut pos);
                input.select_region(pos, pos);
            },
            None => ()
        };


        gtk::glib::Propagation::Proceed
    };

    eck.connect_key_pressed(key_handler);
    application_window.add_controller(eck);
}

pub fn setup_on_clock_tick(launcher: Arc<Mutex<Launcher>>) {
    let on_tick =  move || -> glib::ControlFlow {
        let launcher = launcher.lock().unwrap();
        let clock = launcher.clock.clone();
        let clock = clock.unwrap();
        let clock = clock.borrow();
        clock::set_clock_time(&clock::get_time_str(), &clock);
        glib::ControlFlow::Continue
    };
    glib::timeout_add_seconds_local(1, on_tick);
}