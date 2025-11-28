use crate::launcher::Launcher;
use crate::{Rc, RefCell};
use crate::gobject::{SearchResultBox, SearchEntryIMContext};
use crate::launcher;
use crate::search;
use gtk::prelude::EditableExt;
use gtk::PropagationPhase;
use gtk::prelude::WidgetExt;
use gtk::prelude::IMContextExt;

pub fn attach_screenshot_handlers(launcher: Rc<RefCell<Launcher>>, screenshot_icon: &mut gtk::Image) {
    let ecm = gtk::EventControllerMotion::builder()
        .propagation_phase(gtk::PropagationPhase::Capture).build();
    let gesture_click = gtk::GestureClick::new();

    let launcher_cell_enter = launcher.clone();
    let launcher_cell_focus = launcher.clone();
    let launcher_cell_click = launcher.clone();
    let launcher_cell_focus_notify = launcher;

    let screenshot_enter_handler = move |_: &gtk::EventControllerMotion,  _: f64, _: f64| {
        println!("Screenshot enter handler");
        let launcher = launcher_cell_enter.borrow_mut();
        let screenshot_button = launcher.screenshot_button.clone();
        drop(launcher);
        screenshot_button.grab_focus();
    };

    let screenshot_leave_handler = move |_: &gtk::EventControllerMotion| {
        launcher::focus_text_input(launcher_cell_focus.clone());
    };

    let screenshot_click_handler = move |
        _gc: &gtk::GestureClick, _: i32, _: f64, _: f64| {

        let launcher = launcher_cell_click.borrow_mut();
        let screenshot_button = launcher.screenshot_button.clone();
        drop(launcher);
        screenshot_button.grab_focus();
        let mut launcher = launcher_cell_click.borrow_mut();
        launcher.launch_selected_application();
        drop(launcher);
        launcher::hide_window(launcher_cell_click.clone());
    };

    let screenshot_focus_notify_handler = move |_: &gtk::Image| {
        let mut launcher = launcher_cell_focus_notify.borrow_mut();
        launcher.selected_search_idx = Some(-1);
    };

    ecm.connect_enter(screenshot_enter_handler);
    ecm.connect_leave(screenshot_leave_handler);
    screenshot_icon.connect_has_focus_notify(screenshot_focus_notify_handler);
    gesture_click.connect_pressed(screenshot_click_handler);

    screenshot_icon.add_controller(ecm);
    screenshot_icon.add_controller(gesture_click);
}

pub fn attach_window_key_handler(
        application_window: &mut gtk::ApplicationWindow, 
        launcher_cell: Rc<RefCell<Launcher>>) {

    let eck = gtk::EventControllerKey::builder()
        .propagation_phase(PropagationPhase::Capture).build();

    let key_handler = move |
            _: &gtk::EventControllerKey, key: gdk::Key, _: u32, _: gdk::ModifierType| -> gtk::glib::Propagation {
        match key {
            gdk::Key::Escape => {
                println!("Hiding window");
                launcher::hide_window(launcher_cell.clone());
                return gtk::glib::Propagation::Stop;
            },
            gdk::Key::Return => {
                launcher::handle_enter_key(launcher_cell.clone());
            },
            gdk::Key::Down => {
                launcher::scroll_search_results_down(launcher_cell.clone());
            },
            _ => {
                if let Some(character) = key.to_unicode() {
                    println!("keyboard unicode");
                    if launcher::focus_text_input(launcher_cell.clone()) {

                        // search bar widget never receives key press because it was fired
                        // on some other widget. So this key needs to be inserted manually.
                        let launcher = launcher_cell.borrow();
                        let input_buffer = launcher.input_buffer.clone().unwrap();
                        let search_bar = launcher.search_bar.clone();
                        drop(launcher);
                        let input_bufer = input_buffer.borrow();
                        let pos = input_buffer.borrow().length() as i32;
                        // search_bar.insert_text(&character.to_string(), &mut pos);
                    }
                }
            },
        };

        gtk::glib::Propagation::Proceed
    };

    eck.connect_key_pressed(key_handler);
    application_window.add_controller(eck);
}

pub fn attach_result_box_handlers(
        launcher_cell: Rc<RefCell<Launcher>>, 
        result_box: &mut SearchResultBox, 
        frame_idx: usize) {

    let gesture_click = gtk::GestureClick::builder()
        .propagation_phase(PropagationPhase::Capture).build();
    let ecm = gtk::EventControllerMotion::builder()
        .propagation_phase(PropagationPhase::Capture).build();

    let launcher_cell_gc = launcher_cell.clone();

    gesture_click.connect_pressed(move |_, _, _, _| {
        println!("gesture_click handler {frame_idx}");
        let mut launcher = launcher_cell_gc.borrow_mut();
        if launcher.search_result_frames[frame_idx].has_focus() {
            launcher.launch_selected_application();
            drop(launcher);
        } else {
            let frame = launcher.search_result_frames[frame_idx].clone();
            drop(launcher);
            frame.grab_focus();
        }
        launcher::hide_window(launcher_cell_gc.clone());
    });

    let launcher_cell_ecm = launcher_cell.clone();
    ecm.connect_enter(move |_, _, _| { 
        println!("ecm enter result frame {frame_idx}");
        launcher::handle_result_box_hovered(launcher_cell_ecm.clone(), frame_idx);
    });

    result_box.add_controller(gesture_click);
    result_box.add_controller(ecm);

    let launcher_cell_focus = launcher_cell.clone();

    result_box.connect_has_focus_notify(move |f| {
        println!("result frame focus {frame_idx}");
        let mut launcher = launcher_cell_focus.borrow_mut();
        launcher.selected_search_idx = Some(f.get().idx_in_container.try_into().unwrap());
    });
}

pub fn attach_search_bar_handlers(
        launcher_cell: Rc<RefCell<Launcher>>, 
        search_bar: &mut gtk::Entry) {

    let ec = gtk::EventControllerKey::builder()
        .name("im_controller")
        .propagation_phase(PropagationPhase::Capture).build();
    let im_context = SearchEntryIMContext::new();
    let im_simple = gtk::IMContextSimple::new(); 
    // ec.set_im_context(Some(&im_context));
    im_context.set_use_preedit(true);
    
    let launcher_cell_buffer_changed = launcher_cell.clone();
    search_bar.connect_changed(move |buffer| {
        println!("Search bar changed");
        let buffer = buffer.text().to_string();
        let launcher_cell = launcher_cell_buffer_changed.clone();
        let mut launcher = launcher_cell.borrow_mut();
        let search_results = search::refetch_results(&mut launcher.search_context, buffer);

        // in case the mouse cursor is on a result box while they type, disable stealing cursor focus
        launcher.disable_motion_events(); // will be re-enabled next time a motion event is triggered.
        search::display_search_results(&mut launcher, search_results);
    });

    let launcher_cell_focus = launcher_cell;
    search_bar.connect_has_focus_notify(move |_| {
        println!("search_bar connect_has_focus_notify");
        let mut launcher = launcher_cell_focus.borrow_mut();
        launcher.selected_search_idx = None;
    });

    search_bar.add_controller(ec);
}