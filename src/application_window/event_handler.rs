use crate::launcher::{LAUNCHER, clock};
use gtk::prelude::EditableExt;

pub fn screenshot_enter_handler(_ec: &gtk::EventControllerMotion, _: f64, _: f64) {
    unsafe { LAUNCHER.select_screenshot_button(); }
}

pub fn screenshot_leave_handler(_ec: &gtk::EventControllerMotion) {
    unsafe { LAUNCHER.focus_text_input(); }
}

pub fn screenshot_click_handler(_gc: &gtk::GestureClick, _: i32, _: f64, _: f64) {
    unsafe {
        LAUNCHER.select_screenshot_button();
        LAUNCHER.launch_selected_application();
        LAUNCHER.hide_window();
    }
}

pub fn key_handler(
        _: &gtk::EventControllerKey,
        key: gdk::Key,
        _: u32, 
        _: gdk::ModifierType) -> gtk::glib::Propagation {

    println!("key {}", key);
    
    unsafe {
        match key {
            gdk::Key::Escape => LAUNCHER.hide_window(),
            gdk::Key::Return => LAUNCHER.handle_enter_key(),
            gdk::Key::Down => LAUNCHER.scroll_search_results_down(),
            _ => ()
        };

        match LAUNCHER.selected_search_idx {
            Some(_) => (),
            None => return gtk::glib::Propagation::Proceed,
        };

        match key {
            gdk::Key::BackSpace => {
                LAUNCHER.focus_text_input();
                let input = LAUNCHER.text_input.clone().unwrap();
                let pos = (*LAUNCHER.search_context.clone().unwrap()).borrow().buf.len() as i32;
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
                LAUNCHER.focus_text_input();
                let input = LAUNCHER.text_input.clone().unwrap();
                let mut pos = (*LAUNCHER.search_context.clone().unwrap()).borrow().buf.len() as i32;
                input.insert_text(
                   &character.to_string(), 
                   &mut pos);
                input.select_region(pos, pos);
            },
            None => ()
        };
    }

    gtk::glib::Propagation::Proceed
}

pub fn on_clock_tick() -> glib::ControlFlow {
    unsafe {
        let clock = LAUNCHER.clock.clone();
        let clock = clock.unwrap();
        let clock = clock.borrow();
        clock::set_clock_time(&clock::get_time_str(), &clock);
    }
    glib::ControlFlow::Continue
}