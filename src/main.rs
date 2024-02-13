
use std::io::Read;
use std::path::{Path, PathBuf};
use glib::subclass::shared::RefCounted;
use std::os::fd::AsRawFd;
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

mod search;
mod search_buffer_imp;
mod xdg_desktop_entry;
use xdg_desktop_entry::XdgDesktopEntry;
use search_buffer_imp::SearchEntry;

use inotify::{
    Inotify,
    WatchMask,
};


struct Launcher {
    state: State,
    done_init: bool,
    search_input_buffer: Option<SearchEntry>,
    css_provider: Option<(
        Box<PathBuf>, 
        Arc<File>, 
        Rc<gtk::CssProvider>)>,
    fifo_path: [i8; 2000],
}

#[derive(Copy, Clone, Debug)]
enum State {
    Hidden,
    Visible,
    NotStarted
}

static  mut launcher: Launcher = Launcher { 
	state: State::NotStarted, 
    search_input_buffer: None,
	done_init: false,
    css_provider: None,
    fifo_path: ['\0' as i8; 2000],
}; 

thread_local! {
    static WINDOW: RefCell<Option<gtk::ApplicationWindow>> = RefCell::new(None);
}

fn get_time_str() -> String {
    let date_time  =  chrono::offset::Local::now();
    let formatted = format!("{}", date_time.format("%a %d/%B %Y %H:%M:%S"));
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


fn reload_css() {
    println!("reloading css...");
    unsafe {
        match &launcher.css_provider {
            Some((path, file, provider)) => 
                provider.load_from_path((*path).as_path()),
            None => ()
        };
    }
}

use libc::{c_void, mkfifo, fdopen, fclose, read, fprintf, 
    close, fgets, open, write, O_RDONLY, O_WRONLY, O_NONBLOCK};
unsafe fn startup(application: &gtk::Application) {
    // kill -9 $(ps -aux | grep generic | head -n 1 | tr -s ' ' | cut -d ' ' -f 2)
    println!("Starting up...");

    let mut css_path = glib::user_config_dir();
    css_path.push("generic_launcher/launcher.css");
    let css_file = File::open(css_path.clone()).unwrap();
    let css_file = Arc::new(css_file);

    let w = gtk::ApplicationWindow::new(application);
    let action_close = gio::ActionEntry::builder("close")
        .activate(|w: & gtk::ApplicationWindow, _, _| {
            w.close();
        })
        .build();
    w.add_action_entries([action_close]);


    let provider = gtk::CssProvider::new();
    launcher.css_provider = Some((Box::new(css_path.clone()), css_file, provider.into()));
    match &launcher.css_provider {
        Some((path, file, provider)) =>  {

            let mut pipe_path = css_path.clone();
            pipe_path.set_extension(&"pipe");
            let mut j = 0;
            for (i, c) in pipe_path.to_str().unwrap().chars().enumerate() {
                launcher.fifo_path[i] = c as i8;
                j=i+1;
            }
            launcher.fifo_path[j]= '\0' as i8;
            mkfifo(launcher.fifo_path.as_ptr() as *const i8, 0o666);

            let open_pipe = move |flags| {
                let fd = libc::open(
                    launcher.fifo_path.as_ptr() as *const i8, 
                    flags);
                if fd < 0 {
                    println!("{:?}", &std::io::Error::last_os_error());
                    todo!("err");
                }
                let buffer:  [c_char; 20] = [0; 20];
                (fd, buffer)
            };
            let pipe_box = Box::new(open_pipe.clone());

            match glib::ThreadPool::shared(Some(1)) {
                Err(..) => todo!(),
                Ok(threadpool) => {
                    threadpool.push(move || {
                        std::thread::spawn(move || {
                            println!("{:?}", (*path).as_ref());
                            let mut inotify = Inotify::init().expect("Error while initializing inotify instance");
                            inotify.watches().add((*path).as_ref(), WatchMask::MODIFY | WatchMask::CLOSE)
                                .expect("Failed to add file watch");
                            let (fd, buffer) = pipe_box(O_WRONLY); 
                            let mut buffer2 = [0; 1024];
                            loop {
                                'outer: { 
                                    match inotify.read_events_blocking(&mut buffer2) {
                                        Ok(events) => {
                                            println!("inotify event");

                                            for event in events {
                                                if !matches!(event.mask, inotify::EventMask::CLOSE_NOWRITE) {
                                                    libc::write(fd, buffer.as_ptr() as *mut c_void, 1);
                                                    break 'outer  
                                                }
                                            }
                                        },
                                        Err(error) => {
                                            println!("inotify err: {:?}", error);
                                        }
                                    }
                                }
                            };
                        })
                    });
                }
            };
            
            let (fd, mut buffer) = open_pipe(O_RDONLY);
            glib::source::unix_fd_add_local(
                fd, 
                glib::IOCondition::IN, move |_, d| {
                    let bytes_read = libc::read(fd, buffer.as_ptr() as *mut c_void, 20); 
                    println!("bytes_read {:?}", bytes_read);
                    if bytes_read == 0 {
                        println!("{:?}", &std::io::Error::last_os_error());
                    }
                    let contents = format!("{:?}", String::from_utf8(
                        buffer.to_vec().iter().map(|i| *i as u8).collect()));
                    print!("{}", contents);
                    reload_css();

                    glib::ControlFlow::Continue
                }
            );

            gtk::style_context_add_provider_for_display(
                &gdk::Display::default().expect("Could not connect to a display."),
                provider.as_ref(),
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
            reload_css(); 
        }
        None => ()
    };

    let mut buffer = search_buffer_imp::SearchEntryBuffer::new();
    buffer.context.borrow_mut().set_desktop_files(search::get_xdg_desktop_entries());
    let input_field = gtk::Entry::builder().xalign(0.5)
        .buffer(&SearchEntry::new(buffer)).build();
    input_field.set_halign(gtk::Align::Center);
    let context = input_field.style_context();
    context.add_class("input_field");
    w.init_layer_shell();

    w.set_layer(Layer::Overlay);
    // w.auto_exclusive_zone_enable(); for persistent topbar
    w.set_margin(Edge::Left, 0);
    w.set_margin(Edge::Right, 0);
    w.set_margin(Edge::Top, 0);

    let anchors = [
        (Edge::Left, true),
        (Edge::Right, true),
        (Edge::Top, false),
        (Edge::Bottom, false),
    ];

    for (anchor, state) in anchors {
        w.set_anchor(anchor, state);
    }
    let root = gtk::Box::new(gtk::Orientation::Vertical, 9);
    let result_box = gtk::Box::new(gtk::Orientation::Vertical, 5);

    let mut result_frames: Vec<gtk::Frame> = Vec::new();

    let mouse_controller = gtk::EventControllerMotion::new();
    mouse_controller.connect_motion(listbox_hover_handler);
    // mouse_controller.connect_stylus_only_notify(listbox_hover_handler_new);
    result_box.add_controller(mouse_controller);

    for i in 0..5 {
        let label = format!("label {}", i);
        let frame = gtk::Frame::new(Some(&label));
        let context = frame.style_context();
        context.add_class("result_frame");
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
    let context = clock.style_context();
    context.add_class("clock");
    clock.set_text(&get_time_str());

    root.style_context();
    context.add_class("root");
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
    WINDOW.replace(Some(w));

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