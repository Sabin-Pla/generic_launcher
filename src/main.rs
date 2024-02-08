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
use gio::prelude::*;
use gtk::prelude::*;
use gtk4_layer_shell::{KeyboardMode, Edge, Layer, LayerShell};
use std::ffi::CStr;
use std::os::fd::IntoRawFd;
use std::fs;
use std::os::raw::c_char;
use std::ffi::CString;
use std::fs::File;
use gtk::PropagationPhase;
use std::time::{Duration, SystemTime};

static  mut launcher: Launcher = Launcher { 
	state: State::NotStarted, 
	done_init: false
}; 

thread_local! {
    static WINDOW: RefCell<Option<gtk::ApplicationWindow>> = RefCell::new(None);
}

fn get_time_str() -> String {
    let date_time  =  chrono::offset::Local::now();
    let formatted = format!("{}", date_time.format("%d/%m/%Y %H:%M"));
    formatted
}

fn key_handler(ec: &gtk::EventControllerKey, 
        key: gdk::Key, _: u32, _: gdk::ModifierType) -> gtk::glib::Propagation {
    println!("key {}", key);
    if key == gdk::Key::Escape {
        unsafe {
            WINDOW.with( |w| {
                    let mut w = (*w).borrow_mut();
                    let w = w.as_mut().unwrap();
                    w.hide();
                    launcher.state = State::Hidden;
                }
            )
        }
    }
    gtk::glib::Propagation::Proceed
}

fn handle_mouse_click(_: &gtk::GestureClick, _n_press: i32, _x: f64,  _y: f64) {
    println!("click  {_n_press} {_x} {_y}");
}

fn listbox_hover_handler(_: &gtk::EventControllerMotion, _x: f64, _y: f64) {
    println!("hover  {_x} {_y}");
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

    let provider = gtk::CssProvider::new();
    let mut css_path = glib::user_config_dir();
    css_path.push("generic_launcher/launcher.css");
    provider.load_from_path(css_path.as_path());

    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

    w.add_action_entries([action_close]);
    let input_field = gtk::Entry::new();
    w.init_layer_shell();

    // consider https://gtk-rs.org/gtk4-rs/git/docs/gdk4/prelude/trait.SurfaceExt.html connect_leave_monitor workaround
    // also note TopLevelExt  focus(&self, timestamp: u32
    //  inhibit_system_shortcuts for league wrapper
    w.set_layer(Layer::Overlay);
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
    let root = gtk::Box::new(gtk::Orientation::Vertical, 5);
    let result_box = gtk::Box::new(gtk::Orientation::Vertical, 5);

    let mut result_frames: Vec<gtk::Frame> = Vec::new();

    let mouse_controller = gtk::EventControllerMotion::new();
    mouse_controller.connect_motion(listbox_hover_handler);
    // mouse_controller.connect_stylus_only_notify(listbox_hover_handler_new);
    result_box.add_controller(mouse_controller);

    for i in 0..5 {
        let label = format!("label {}", i);
        let frame = gtk::Frame::new(Some(&label));
        let gcc_right_click = gtk::GestureClick::builder()
            .button(1).propagation_phase(PropagationPhase::Capture).build();
        gcc_right_click.connect_pressed(handle_mouse_click);
        frame.add_controller(gcc_right_click);
        result_frames.push(frame);
    }

    for f in result_frames  {
        result_box.append(&f);
    }

    let ec = gtk::EventControllerKey::builder()
        .propagation_phase(PropagationPhase::Capture).build();
    ec.connect_key_pressed(key_handler);    //ec.forward(&input_field);
    input_field.add_controller(ec);

    let clock = gtk::Label::default();
    clock.set_text(&get_time_str()  );
    root.append(&clock);

    let tick = move || { 
        clock.set_text(&get_time_str());
        glib::ControlFlow::Continue
    };

    // executes the closure once every second
    glib::timeout_add_seconds_local(1, tick);
    
    root.append(&input_field);
    root.append(&result_box);
    w.set_child(Some(&root));
    input_field.grab_focus_without_selecting();
    w.set_keyboard_mode(KeyboardMode::Exclusive);
    w.show();

    //w.add clock https://github.com/gtk-rs/gtk4-rs/blob/master/examples/clock/main.rs
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