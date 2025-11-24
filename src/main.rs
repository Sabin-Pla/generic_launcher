
mod application_window;
mod gobject;
mod launcher;
mod search;
mod user_config;
mod utils;
mod xdg_desktop_entry;


use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::ffi::OsStr;
pub use std::collections::HashMap;
pub use std::path::{Path, PathBuf};
pub use std::vec::Vec;
pub use std::time::Duration;

use crate::launcher::Launcher;

use gio::prelude::*;
use gtk::prelude::*;
use gtk4_layer_shell::LayerShell;

use gobject::{SearchEntryBuffer};
use launcher::State;
use user_config::ApplicationSettings;



thread_local! {
    static WINDOW: RefCell<Option<gtk::ApplicationWindow>> = RefCell::new(None);
}

unsafe fn activate(_application: &gtk::Application, launcher_cell: Rc<RefCell<Launcher>>) {
    // this function is called whenever the application is 'activated' (reopened after being dismissed)
	println!("Activating...");
    let launcher_cell_clock = launcher_cell.clone();
    let mut launcher = launcher_cell.borrow_mut();

	WINDOW.with( |application_window| {
		let mut application_window = (*application_window).borrow_mut();
		let application_window = application_window.as_mut().unwrap();
		match launcher.state {
			State::NotStarted => panic!("cannot activate; not started"),
			State::Visible => {
    			println!("Hiding");
                drop(launcher);
    			application_window.set_visible(false);
                println!("hide.");
                let mut launcher = launcher_cell.borrow_mut();
    			launcher.state = State::Hidden;
			},
			State::Hidden => { 
                // todo!("get state from user config");
    			//	application_window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::OnDemand);
                let clock = unsafe {
                    launcher.clock.clone().expect("Clock not initialized")
                };
                let clock = clock.borrow();
                let display = clock.display();

                drop(launcher);
                application_window.set_visible(true);

                let surface = application_window.surface().unwrap();
                let display = display.monitor_at_surface(&surface);
                let rect =  display.unwrap().geometry();
                let (width, height) = (rect.width(), rect.height());
                println!("monitor: {width} {height}");

                println!("---");
                let mut launcher = launcher_cell.borrow_mut();
                println!("---");
                launcher.current_monitor = Some((width, height));
                drop(launcher);
                println!("set_clock_size()");
                launcher::clock::set_clock_size(application_window, launcher_cell_clock);
                let mut launcher = launcher_cell.borrow_mut();
                println!("clear_search_results()");
                launcher.clear_search_results();
                println!("clear_search_buffer()");
                let text_input = launcher.text_input.clone().unwrap();
                drop(launcher);
                text_input.set_text("");
                println!("focus_text_input()");
                text_input.grab_focus();
                let mut launcher = launcher_cell.borrow_mut();
    			launcher.state = State::Visible;
    		}
		}
	});
}

unsafe fn startup(application: &gtk::Application, launcher_cell: Rc<RefCell<Launcher>>) {
    let application_settings = ApplicationSettings::load();
    println!("Loading application settings: {:?}", application_settings);
    std::fs::create_dir(application_settings.user_config.screenshots_destination_directory.clone());
   
    let mut application_window = application_window::initialize(application);
    application_window::populate(&mut application_window, &application_settings, launcher_cell.clone());
    let mut launcher = launcher_cell.borrow_mut();
    let css_file = Arc::new(application_settings.css_file);
    let provider = gtk::CssProvider::new();
    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

    launcher.css_provider = Some((css_file.clone(), provider.into()));

    drop(launcher);
    utils::hot_reload::attach(
        &css_file.path().expect("Error getting pathbuf for css provider"), launcher_cell.clone());
    let mut launcher = launcher_cell.borrow_mut();
    launcher.reload_css(); 
    launcher.state = launcher::State::Hidden;
    WINDOW.replace(Some(application_window));
}

fn main()  -> gtk::glib::ExitCode {
    unsafe {
        let application = gtk::Application::new(
            Some("www.generic_launcher_example"), Default::default());
        gtk::prelude::GtkApplicationExt::set_accels_for_action(
            &application, "win.close", &["<Ctrl>C"]);
        let launcher = Rc::new(RefCell::new(Launcher::uninitialized()));
        let startup_launcher = launcher.clone();
        application.connect_startup(move |app| {
            startup(app, startup_launcher.clone());
            println!("Done initial startup");
            // activate(app, startup_launcher.clone())
        });
        application.connect_activate(move |app| {
            activate(app, launcher.clone())
        });
        application.run()
    }
}   