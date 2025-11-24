use crate::{Rc, RefCell, HashMap, Mutex};

use gtk::PropagationPhase;
use gtk::prelude::*;
use gtk4_layer_shell::{KeyboardMode, LayerShell};

use crate::launcher;
use crate::launcher::{Launcher, RESULT_ENTRY_COUNT};
use crate::search::SearchContext;
use crate::launcher::clock;
use crate::gobject::{SearchResultBox, SearchResultBoxWidget, SearchEntryIMContext};

use super::event_handler;

pub fn root(application_window: &mut gtk::ApplicationWindow, launcher: Rc<RefCell<Launcher>>, icon_theme: &gtk::IconTheme) {
	let root_box = gtk::Box::new(gtk::Orientation::Vertical, 9);
	let root_style = root_box.style_context();
    root_style.add_class("root");
	root_box.append(&topbar(launcher.clone(), icon_theme));
    root_box.append(&search_bar(application_window, launcher.clone()));
    root_box.append(&search_result_box(launcher));
    application_window.set_child(Some(&root_box));
}

fn topbar(launcher: Rc<RefCell<Launcher>>, icon_theme: &gtk::IconTheme) -> gtk::CenterBox {
	let topbar = gtk::CenterBox::builder()
        .orientation(gtk::Orientation::Horizontal)
        .build();
    topbar.set_center_widget(Some(&clock_box(launcher.clone())));
	topbar.set_end_widget(Some(&screenshot_button(launcher, icon_theme)));
	topbar
}

fn search_bar(application_window: &mut gtk::ApplicationWindow, launcher_cell: Rc<RefCell<Launcher>>) -> gtk::Entry {
    let launcher_cell_focus = launcher_cell.clone();
    let launcher_cell_search_entry = launcher_cell.clone();
    let mut launcher = launcher_cell.borrow_mut();

	let ec = gtk::EventControllerKey::builder()
        .name("im_controller")
        .propagation_phase(PropagationPhase::Capture).build();
    let im_context = SearchEntryIMContext::new();
    let im_simple = gtk::IMContextSimple::new(); 
    // ec.set_im_context(Some(&im_context));

    use crate::{xdg_desktop_entry, SearchEntryBuffer};
    let xdg_desktop_entries = xdg_desktop_entry::get_xdg_desktop_entries();
    
    let desktop_entries = Rc::new(xdg_desktop_entries.0);
    let custom_launchers = Rc::new(xdg_desktop_entries.1);
    launcher.user_desktop_files = Some(desktop_entries.clone());
    launcher.search_context.user_desktop_files = desktop_entries.clone();
    launcher.custom_launchers = Some(custom_launchers);
    let search_entry_buffer = SearchEntryBuffer::new();
    let buffer_refcell = RefCell::new(search_entry_buffer);
    launcher.input_buffer = Some(buffer_refcell.clone());
    let buffer = buffer_refcell.borrow();


    drop(launcher); // gtk entry builder.buffer() tries to grab mutex so drop and relock
	let mut search_bar = gtk::Entry::builder().xalign(0.5)
        .buffer(&*buffer).build();
    let mut launcher = launcher_cell.borrow_mut();

    search_bar.set_halign(gtk::Align::Center);
    search_bar.add_controller(ec);
    im_context.set_use_preedit(true);
    let context = search_bar.style_context();
    context.add_class("input-field");

    search_bar.set_focusable(true);
    search_bar.grab_focus_without_selecting();
    application_window.set_keyboard_mode(KeyboardMode::Exclusive);
    launcher.text_input = Some(Rc::new(search_bar.clone()));
    let search_bar = &mut search_bar;

    drop(launcher); // accessing buffer locks mutex...
    search_bar.set_placeholder_text(Some("Applications"));
    let mut launcher = launcher_cell.borrow_mut();

    search_bar.connect_has_focus_notify(move |_f| {
        let mut launcher = launcher_cell_focus.borrow_mut();
        launcher.selected_search_idx = None;
    });
    
    search_bar.set_has_frame(true);
    launcher.clear_search_results();
    search_bar.clone()
}

fn search_result_box(launcher_cell: Rc<RefCell<Launcher>>) -> gtk::Box {

	let result_box = gtk::Box::new(gtk::Orientation::Vertical, 5);

    let mut result_frames: Vec<SearchResultBox> = Vec::new();

    for i in 0..RESULT_ENTRY_COUNT {
        let result_box = SearchResultBoxWidget::from(i);
        let result_box = SearchResultBox::new(result_box);
        result_box.set_focusable(true);
        result_box.set_focus_on_click(true);
        gtk::prelude::ButtonExt::set_label(&result_box, &"");
        let gesture_click = gtk::GestureClick::builder()
            .propagation_phase(PropagationPhase::Capture).build();
        let ecm = gtk::EventControllerMotion::builder()
            .propagation_phase(PropagationPhase::Capture).build();

        let launcher_cell_gc = launcher_cell.clone();

        gesture_click.connect_pressed(move |_, _, _, _| {
            let mut launcher = launcher_cell_gc.borrow_mut();
            if launcher.search_result_frames[i].has_focus() {
                launcher.launch_selected_application();
                drop(launcher);
                launcher::hide_window(launcher_cell_gc.clone());
            } else {
                launcher.search_result_frames[i].grab_focus();
            }
        });

        let launcher_cell_ecm = launcher_cell.clone();
    
        ecm.connect_enter(move |_, _, _| { 
            let mut launcher = launcher_cell_ecm.borrow_mut();
            launcher.handle_hovered(i) 
        });
        result_box.add_controller(gesture_click);
        result_box.add_controller(ecm);

        let launcher_cell_focus = launcher_cell.clone();

        result_box.connect_has_focus_notify(move |f| {
            let mut launcher = launcher_cell_focus.borrow_mut();
            launcher.selected_search_idx = Some(
                f.get().idx_in_container.try_into().unwrap());
        });
      
        let context = result_box.style_context();
        context.add_class("result-box");
        result_frames.push(result_box.into());
    }

    for f in &result_frames  {
        result_box.append(f);
    }
    let mut launcher = launcher_cell.borrow_mut();
    launcher.search_result_frames = result_frames;
    result_box
}

fn screenshot_button(launcher_cell: Rc<RefCell<Launcher>>, icon_theme: &gtk::IconTheme) -> gtk::Image {
    event_handler::attach_screenshot_handlers(launcher_cell.clone());
    let launcher_cell_connect_focus = launcher_cell.clone();
    let mut launcher = launcher_cell.borrow_mut();

    // todo!("set the sizes dynamically");
	let screenshot_paintable = icon_theme.lookup_icon(
        "adwaita-applets-screenshooter-symbolic", &[], 
        32, 1, 
        gtk::TextDirection::None, 
        gtk::IconLookupFlags::PRELOAD);
    let screenshot_icon = gtk::Image::from_paintable(Some(&screenshot_paintable));
    screenshot_icon.set_icon_size(gtk::IconSize::Large);
    screenshot_icon.set_focusable(true);

    
    screenshot_icon.connect_has_focus_notify(move |_| {
        let mut launcher = launcher_cell_connect_focus.borrow_mut();
        launcher.selected_search_idx = Some(-1);
    });

    // screenshot_icon.add_controller(ecm);
    let screenshot_style = screenshot_icon.style_context();
    screenshot_style.add_class("screenshot-button");
    launcher.screenshot_button = Some(Rc::new(screenshot_icon.clone()));
    let gesture_click = gtk::GestureClick::new();
    gesture_click.connect_pressed(event_handler::screenshot_click_handler);
    screenshot_icon.add_controller(gesture_click);
    screenshot_icon
}

fn clock_box(launcher_cell: Rc<RefCell<Launcher>>) -> gtk::Box {
    let mut launcher = launcher_cell.borrow_mut();
	let clock_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let clock = gtk::Label::default();
    launcher.clock = Some(Rc::new(RefCell::new(clock.clone())));
    launcher.clock_sizes = Some(HashMap::new());
    clock_box.append(&clock);
    let clock_style = clock.style_context();
    clock_style.add_class("clock");
    clock.set_xalign(0.0);
    clock::set_clock_time(&clock::get_time_str(), &clock);
    event_handler::setup_on_clock_tick(launcher_cell.clone());
    clock_box
}