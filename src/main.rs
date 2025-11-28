mod application_window;
mod gobject;
mod launcher;
mod search;
mod user_config;
mod utils;
mod xdg_desktop_entry;

use std::cell::RefCell;
use std::ffi::OsStr;
pub use std::path::{Path, PathBuf};
use std::rc::Rc;
pub use std::time::Duration;
pub use std::vec::Vec;

use crate::launcher::Launcher;

use gio::prelude::*;
use gtk::prelude::*;
use gtk4_layer_shell::LayerShell;

use gobject::SearchEntryBuffer;
use launcher::State;
use user_config::ApplicationSettings;

thread_local! {
    static WINDOW: RefCell<Option<gtk::ApplicationWindow>> = RefCell::new(None);
}

unsafe fn activate(_application: &gtk::Application, launcher_cell: Rc<RefCell<Launcher>>) {
    // this function is called whenever the application is 'activated' (reopened after being dismissed)
    let mut launcher = launcher_cell.borrow_mut();

    WINDOW.with(|application_window| {
        let mut application_window = (*application_window).borrow_mut();
        let application_window = application_window.as_mut().unwrap();
        match launcher.state {
            State::NotStarted => panic!("cannot activate; not started"),
            State::Visible => {
                println!("Hiding launcher");
                drop(launcher);
                application_window.set_visible(false);
                let mut launcher = launcher_cell.borrow_mut();
                *launcher.current_monitor.borrow_mut() = None;
                launcher.state = State::Hidden;
            }
            State::Hidden => {
                application_window.set_visible(true);
                let surface = application_window.surface().unwrap();
                let display = gtk::prelude::WidgetExt::display(application_window);
                let display = display.monitor_at_surface(&surface);
                let rect = display.unwrap().geometry();
                let (monitor_width, monitor_height) = (rect.width(), rect.height());
                *launcher.current_monitor.borrow_mut() = Some((monitor_width, monitor_height));

                launcher.clear_search_results();
                let search_bar = launcher.search_bar.clone();
                drop(launcher);
                search_bar.set_text("");
                search_bar.grab_focus();
                let mut launcher = launcher_cell.borrow_mut();
                launcher.state = State::Visible;
            }
        }
    });
}

unsafe fn startup(application: &gtk::Application, launcher_cell: Rc<RefCell<Launcher>>) {
    let application_settings = ApplicationSettings::load();
    println!("Loading application settings: {:?}", application_settings);
    let _ = std::fs::create_dir(
        application_settings
            .user_config
            .screenshots_destination_directory
            .clone(),
    );

    let mut application_window = application_window::initialize(application);
    application_window::populate(
        &mut application_window,
        &application_settings,
        launcher_cell.clone(),
    );

    // todo!("get state from user config");
    application_window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive);

    let mut launcher = launcher_cell.borrow_mut();
    let css_file = std::sync::Arc::new(application_settings.css_file);
    let provider = gtk::CssProvider::new();
    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    launcher.css_provider = Some((css_file.clone(), provider.into()));

    drop(launcher);
    utils::hot_reload::attach(
        &css_file
            .path()
            .expect("Error getting pathbuf for css provider"),
        launcher_cell.clone(),
    );
    let mut launcher = launcher_cell.borrow_mut();
    launcher.reload_css();
    launcher.state = launcher::State::Hidden;
    WINDOW.replace(Some(application_window));
}

fn main() -> Result<gtk::glib::ExitCode, glib::error::BoolError> {
    unsafe {
        gtk::init()?;
        let application =
            gtk::Application::new(Some("www.generic_launcher_example"), Default::default());
        gtk::prelude::GtkApplicationExt::set_accels_for_action(
            &application,
            "win.close",
            &["<Ctrl>C"],
        );
        let launcher = Rc::new(RefCell::new(Launcher::uninitialized()));
        let startup_launcher_cell = launcher.clone();
        application.connect_startup(move |app| {
            startup(app, startup_launcher_cell.clone());
        });
        application.connect_activate(move |app| activate(app, launcher.clone()));
        Ok(application.run())
    }
}
