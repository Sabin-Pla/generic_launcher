
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
use std::ffi::OsStr;
pub use std::collections::HashMap;
pub use std::path::{Path, PathBuf};
pub use std::vec::Vec;
pub use std::time::Duration;

use gio::prelude::*;
use gtk::prelude::*;
use gtk4_layer_shell::LayerShell;

use gobject::{SearchEntryBuffer};
use launcher::{State, LAUNCHER};
use user_config::ApplicationSettings;



thread_local! {
    static WINDOW: RefCell<Option<gtk::ApplicationWindow>> = RefCell::new(None);
}

unsafe fn activate(_application: &gtk::Application) {
    // this function is called whenever the application is 'activated' (reopened after being dismissed)
	println!("Activating...");
	if LAUNCHER.done_init {
		WINDOW.with( |application_window| {
			let mut application_window = (*application_window).borrow_mut();
  			let application_window = application_window.as_mut().unwrap();
  			match LAUNCHER.state {
  				State::NotStarted => panic!("cannot activate; not started"),
  				State::Visible => {
  					println!("Hiding");
  					application_window.set_visible(false);
  					LAUNCHER.state = State::Hidden;
  				},
  				State::Hidden => { 
  					println!("Showing window");
                    // todo!("get state from user config");
  					application_window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive);
  					application_window.set_visible(true);
                    let clock = unsafe {
                        LAUNCHER.clock.clone().expect("Clock not initialized")
                    };
                    let clock = clock.borrow();
                    let display = clock.display();
                    let surface = application_window.surface().unwrap();
                    let display = display.monitor_at_surface(&surface);
                    let rect =  display.unwrap().geometry();
                    let (width, height) = (rect.width(), rect.height());
                    println!("monitor: {width} {height}");
                    LAUNCHER.current_monitor = Some((width, height));
                    LAUNCHER.set_clock_size(application_window);
                    LAUNCHER.clear_search_results();
                    LAUNCHER.clear_search_buffer();
                    LAUNCHER.focus_text_input();
  					LAUNCHER.state = State::Visible;
  				}
  			}
		});
	} else {
        println!("not yet initialized...");
    }
	LAUNCHER.done_init = true;
}

unsafe fn startup(application: &gtk::Application) {
    let application_settings = ApplicationSettings::load();
    println!("Loading application settings: {:?}", application_settings);
    std::fs::create_dir(application_settings.user_config.screenshots_destination_directory.clone());
        
    let mut application_window = application_window::initialize(application);
    application_window::populate(&mut application_window, &application_settings, &mut LAUNCHER);

    let css_file = Arc::new(application_settings.css_file);

    let provider = gtk::CssProvider::new();
    LAUNCHER.css_provider = Some((css_file, provider.into()));
    match &LAUNCHER.css_provider {
        Some((file, provider)) =>  {
            utils::hot_reload::attach(
                &file.path().expect("Error getting pathbuf for css provider"), &mut LAUNCHER);
            gtk::style_context_add_provider_for_display(
                &gdk::Display::default().expect("Could not connect to a display."),
                provider.as_ref(),
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
            LAUNCHER.reload_css(); 
        }
        None => ()
    };


    WINDOW.replace(Some(application_window));
}

fn main()  -> gtk::glib::ExitCode {
    unsafe {
        LAUNCHER.state = State::Hidden;
        let application = gtk::Application::new(
            Some("www.generic_launcher_example"), Default::default());
        gtk::prelude::GtkApplicationExt::set_accels_for_action(
            &application, "win.close", &["<Ctrl>C"]);
        application.connect_startup(|app| {
            startup(app);
            activate(app)
        });
        application.connect_activate(|app| {
            activate(app) 
        });
        application.run()
    }
}   