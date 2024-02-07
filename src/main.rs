use gtk::subclass::prelude::ButtonImpl;
use gtk::subclass::prelude::WidgetImpl;
use gtk::subclass::prelude::ObjectSubclass;
use gtk::subclass::prelude::ObjectImpl;
use std::sync::OnceLock;
use glib::subclass::Signal;


use std::sync::Mutex;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use libc::{O_WRONLY, O_RDONLY, O_NONBLOCK};
use gio::prelude::*;
use gtk::prelude::*;
use gtk4_layer_shell::{KeyboardMode, Edge, Layer, LayerShell};
use std::ffi::CStr;

use std::os::fd::IntoRawFd;
use std::fs;
use std::os::raw::c_char;
use std::ffi::CString;
use libc::{c_void, mkfifo, fdopen, fclose, read, fprintf, close, fgets, open, write};
use std::fs::File;


static  mut launcher: Launcher = Launcher { 
	state: State::NotStarted, 
	done_init: false
}; 

thread_local! {
    static WINDOW: RefCell<Option<gtk::ApplicationWindow>> = RefCell::new(None);
}


unsafe fn activate(application: &gtk::Application) {
	println!("Activating...");
	if launcher.done_init {
		WINDOW.with( |w| {
			let mut w = (*w).borrow_mut();
  			let w = w.as_mut().unwrap();
  			match launcher.state {
  				State::NotStarted => panic!("cannot activate; not started"),
  				State::Visible => {
  					println!("Hiding");
  					w.hide();
  					launcher.state = State::Hidden;
  				},
  				State::Hidden => { 
  					println!("Showing window");
  					w.set_keyboard_mode(KeyboardMode::Exclusive);
  					w.show();
  					launcher.state = State::Visible;
  				}
  			}
		});
	}
	launcher.done_init = true;
}


unsafe fn startup(application: &gtk::Application) {
    println!("Starting up...");
    let w = gtk::ApplicationWindow::new(application);
    let action_close = gio::ActionEntry::builder("close")
        .activate(|w: & gtk::ApplicationWindow, _, _| {
            w.close();
        })
        .build();
    w.add_action_entries([action_close]);
    let input_field = gtk::Entry::new();
    w.init_layer_shell();
    w.set_layer(Layer::Top);
    w.auto_exclusive_zone_enable();
    w.set_margin(Edge::Left, 90);
    w.set_margin(Edge::Right, 90);
    w.set_margin(Edge::Top, 90);

    let anchors = [
        (Edge::Left, false),
        (Edge::Right, false),
        (Edge::Top, false),
        (Edge::Bottom, false),
    ];

    for (anchor, state) in anchors {
        w.set_anchor(anchor, state);
    }

    w.set_child(Some(&input_field));
    input_field.grab_focus_without_selecting();
    w.set_keyboard_mode(KeyboardMode::Exclusive);
    w.show();
    WINDOW.replace(Some(w));

}

struct Launcher {
    state: State,
    done_init: bool
}

#[derive(Copy, Clone, Debug)]
enum State {
    Hidden,
    Visible,
    NotStarted
}

fn main()  -> gtk::glib::ExitCode {
    unsafe {
        launcher.state = State::Visible;
        let application = gtk::Application::new(
            Some("sh.wmww.generic_launcher_example"), Default::default());
        application.set_accels_for_action("win.close", &["<Ctrl>C"]);
        application.connect_startup(|app| {
            startup(app) 
        });
        application.connect_activate(|app| {
            activate(app) 
        });
        application.run()
    }
}   