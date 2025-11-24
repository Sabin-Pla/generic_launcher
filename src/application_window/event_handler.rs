use crate::launcher::{Launcher, clock};
use crate::{Rc, RefCell};
use crate::application_window::event_handler;
use crate::launcher;
use crate::search;
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

pub fn attach_screenshot_handlers(launcher: Rc<RefCell<Launcher>>) {
    let ecm = gtk::EventControllerMotion::builder()
        .propagation_phase(gtk::PropagationPhase::Capture).build();

    let enter_launcher = launcher.clone();
    let screenshot_enter_handler = move |_: &gtk::EventControllerMotion,  _: f64, _: f64| {
        println!("Screenshot enter handler");
        let mut enter_launcher = enter_launcher.borrow_mut();
        enter_launcher.select_screenshot_button();
        println!("---------");
    };

    let screenshot_leave_handler = move |_: &gtk::EventControllerMotion| {
        launcher::focus_text_input(launcher.clone());
    };

    ecm.connect_enter(screenshot_enter_handler);
    ecm.connect_leave(screenshot_leave_handler);
}

pub fn attach_window_key_handler(
        application_window: &mut gtk::ApplicationWindow, 
        launcher_cell: Rc<RefCell<Launcher>>) {

    let launcher_cell_capture = launcher_cell.clone();
    let launcher_cell_target = launcher_cell;

    let eck_target = gtk::EventControllerKey::builder()
        .propagation_phase(PropagationPhase::Bubble).build();
    let eck_capture = gtk::EventControllerKey::builder()
        .propagation_phase(PropagationPhase::Capture).build();

    let key_handler_capture = move |
            _: &gtk::EventControllerKey, key: gdk::Key, _: u32, _: gdk::ModifierType| -> gtk::glib::Propagation {
        match key {
            gdk::Key::Escape => {
                println!("Hiding window");
                launcher::hide_window(launcher_cell_capture.clone());
                return gtk::glib::Propagation::Stop
            },
            gdk::Key::Return => {
                println!("RETURN PRESSED");
                launcher::handle_enter_key(launcher_cell_capture.clone());
                return gtk::glib::Propagation::Proceed
            },
            gdk::Key::Down => {
                launcher::scroll_search_results_down(launcher_cell_capture.clone());
                return gtk::glib::Propagation::Proceed
            },
            gdk::Key::BackSpace => {

                // if this is not here backspace deletes all characters
                let launcher = launcher_cell_capture.borrow();
                match launcher.selected_search_idx {
                    Some(_) => {
                         drop(launcher);
                         launcher::focus_text_input(launcher_cell_capture.clone());
                    },
                    None => {
                        drop(launcher);
                    },
                };
                
                let mut launcher = launcher_cell_capture.borrow_mut();
                let input = launcher.text_input.clone().unwrap();
                let buffer = launcher.input_buffer.clone().unwrap();
                let buffer = buffer.borrow();
                let pos = (*buffer).length() as i32;
                println!("pos {pos}");

                input.delete_text(pos -1, pos);
                let buffer = launcher.input_buffer.as_ref().unwrap();
                let buffer = buffer.borrow().clone();
                let buffer = buffer.text();
                let buffer = buffer.borrow().clone();
                let search_context = &mut launcher.search_context;
                println!("search::text_deleted(search_context, \"{}\")", &buffer);
                let search_results = search::text_deleted(search_context, buffer); 
                search::display_search_results(&mut launcher, search_results);
                input.select_region(pos - 1, pos - 1);
                return gtk::glib::Propagation::Proceed
            },
            _ => ()
        };

        match key.to_unicode() {
            Some('\r')|None => gtk::glib::Propagation::Proceed,
            Some(character) => {
                println!("Processing character |{}|", character as u8);
                launcher::focus_text_input(launcher_cell_capture.clone());
                let mut launcher = launcher_cell_capture.borrow_mut();
                let buffer = launcher.input_buffer.as_ref().unwrap();
                let buffer = buffer.borrow().clone();
                let pos = (buffer).length() as i32;
                let buffer = buffer.text();
                let buffer = buffer.borrow().clone();
                let input = launcher.text_input.clone().unwrap();

                // doesn't actually modify buffer used in widget
                let mut buffer = buffer.to_string().clone();

                buffer.push(character);
                let search_context = &mut launcher.search_context;
                println!("search::text_inserted(search_context, \"{buffer}\")");
                let search_results = search::text_inserted(search_context, buffer); 
                search::display_search_results(&mut launcher, search_results);
                input.select_region(pos, pos);
                gtk::glib::Propagation::Proceed
            }
        }
    };

    eck_capture.connect_key_pressed(key_handler_capture);
    application_window.add_controller(eck_capture);
}

pub fn setup_on_clock_tick(launcher: Rc<RefCell<Launcher>>) {
    let on_tick =  move || -> glib::ControlFlow {
        let launcher = launcher.borrow_mut();
        let clock = launcher.clock.clone();
        let clock = clock.unwrap();
        let clock = clock.borrow();
        clock::set_clock_time(&clock::get_time_str(), &clock);
        glib::ControlFlow::Continue
    };
    glib::timeout_add_seconds_local(1, on_tick);
}