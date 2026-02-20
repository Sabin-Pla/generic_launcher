use std::cell::RefCell;
use std::rc::Rc;

use crate::gobject::{ClockWidget, VolumeControl};
use crate::launcher::{Launcher};
use crate::{SearchEntryBuffer, xdg_desktop_entry};
use gtk::prelude::*;

use super::event_handler;

pub fn root(
    application_window: &mut gtk::ApplicationWindow,
    launcher_cell: Rc<RefCell<Launcher>>,
    icon_theme: &gtk::IconTheme,
) {
    let root_box = gtk::Box::new(gtk::Orientation::Vertical, 9);
    let search_bar = search_bar(launcher_cell.clone());
    root_box.add_css_class("root");
    let topbar = &topbar(launcher_cell.clone(), icon_theme, application_window, &search_bar);
    root_box.append(topbar);
    root_box.append(&search_bar);
    

    let launcher = launcher_cell.borrow();
    let search_result_container = launcher.search_result_container.clone();

    drop(launcher);
    search_result_container.attach_result_box_handlers(event_handler::attach_result_box_handlers, launcher_cell.clone());
    search_result_container.attach_scroll_bar_handler(event_handler::results_scroll_handler, launcher_cell);
    root_box.append(&search_result_container);
    application_window.set_child(Some(&root_box));
}

fn topbar(
        launcher: Rc<RefCell<Launcher>>, 
        icon_theme: &gtk::IconTheme, 
        application_window: &gtk::ApplicationWindow,
        focus_on_panel_hide: &impl IsA<gtk::Widget>) -> gtk::CenterBox {
    let topbar = gtk::CenterBox::builder()
        .orientation(gtk::Orientation::Horizontal)
        .build();

    let monitor_cell = launcher.borrow().current_monitor.clone();
    topbar.set_center_widget(Some(&ClockWidget::new(monitor_cell, application_window, focus_on_panel_hide, &icon_theme)));
    let right = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    right.add_css_class("right-box");

    right.append(&volume_button(launcher.clone(), icon_theme, application_window, focus_on_panel_hide));
    right.append(&screenshot_button(launcher, icon_theme));
    topbar.set_end_widget(Some(&right));
    topbar
}

fn search_bar(launcher_cell: Rc<RefCell<Launcher>>) -> gtk::Entry {
    let mut launcher = launcher_cell.borrow_mut();

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
    let mut search_bar = gtk::Entry::builder().xalign(0.5).buffer(&*buffer).build();

    event_handler::attach_search_bar_handlers(launcher_cell.clone(), &mut search_bar);

    let mut launcher = launcher_cell.borrow_mut();

    search_bar.set_halign(gtk::Align::Center);
    search_bar.add_css_class("input-field");

    search_bar.set_focusable(true);
    search_bar.grab_focus_without_selecting();
    launcher.search_bar = Rc::new(search_bar.clone());
    let search_bar = &mut search_bar;

    drop(launcher);
    search_bar.set_placeholder_text(Some("Applications"));
    search_bar.set_has_frame(true);
    let launcher = launcher_cell.borrow_mut();
    launcher.hide_search_results_container();
    search_bar.clone()
}

fn screenshot_button(
    launcher_cell: Rc<RefCell<Launcher>>,
    icon_theme: &gtk::IconTheme,
) -> gtk::Image {
    let mut launcher = launcher_cell.borrow_mut();

    // todo!("set the sizes dynamically");
    let screenshot_paintable = icon_theme.lookup_icon(
        "adwaita-applets-screenshooter-symbolic",
        &[],
        32,
        1,
        gtk::TextDirection::None,
        gtk::IconLookupFlags::PRELOAD,
    );
    let mut screenshot_icon = gtk::Image::from_paintable(Some(&screenshot_paintable));
    event_handler::attach_screenshot_handlers(launcher_cell.clone(), &mut screenshot_icon);
    screenshot_icon.set_icon_size(gtk::IconSize::Large);
    screenshot_icon.set_focusable(true);

    screenshot_icon.add_css_class("screenshot-button");
    launcher.screenshot_button = Rc::new(screenshot_icon.clone());
    screenshot_icon
}

fn volume_button(
    launcher_cell: Rc<RefCell<Launcher>>,
    icon_theme: &gtk::IconTheme,
    application_window: &gtk::ApplicationWindow,
    focus_on_panel_hide: &impl IsA<gtk::Widget>
) -> VolumeControl {
    let volume_button = VolumeControl::new(icon_theme, &application_window.clone(), focus_on_panel_hide);

    event_handler::attach_volume_handlers(launcher_cell.clone(), volume_button.clone(), application_window.clone());
    volume_button.set_focusable(true);
    volume_button.add_css_class("volume-button");
    volume_button
}
