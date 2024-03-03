
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
use search_buffer_imp::{SearchEntry, SearchEntryBuffer};
use search::SearchContext;

mod search_result_box_impl;
use search_result_box_impl::{SearchResultBoxWidget, SearchResultBox};

use inotify::{
    Inotify,
    WatchMask,
};

pub const RESULT_ENTRY_COUNT: usize = 4;


pub struct Launcher {
    state: State,
    done_init: bool,
    css_provider: Option<(
        Box<PathBuf>, 
        Arc<File>, 
        Rc<gtk::CssProvider>)>,
    fifo_path: [i8; 2000],
    search_result_frames: Vec<SearchResultBox>,
    selected_search_idx: Option<usize>,
    text_input: Option<Rc<gtk::Entry>>,
    user_desktop_files: Option<Rc<Vec<XdgDesktopEntry>>>,
    search_context: Option<Rc<RefCell<SearchContext>>>,
    input_buffer: Option<Rc<SearchEntry>>
}

#[derive(Copy, Clone, Debug)]
enum State {
    Hidden,
    Visible,
    NotStarted
}


static mut launcher: Launcher = Launcher { 
	state: State::NotStarted, 
	done_init: false,
    css_provider: None,
    fifo_path: ['\0' as i8; 2000],
    search_result_frames: vec!(),
    selected_search_idx: None,
    text_input: None,
    user_desktop_files: None,
    search_context: None,
    input_buffer: None
}; 

impl Launcher {
    pub fn clear_search_results(&mut self) {
        for frame in &self.search_result_frames {
            frame.set_label(Some(""));
            frame.set_focusable(false);
            frame.set_visible(false);
        }
    }

    pub fn focus_text_input(&mut self) {
        self.text_input.clone().unwrap().grab_focus_without_selecting();
    }

    fn unfocus_search_result(&mut self, idx_of_focused: usize) {
        println!("not implemented");
        // self.search_result_frames[idx_of_focused];
    }

    pub fn hide_window(&mut self) {
        WINDOW.with( |w| {
                let mut w = (*w).borrow_mut();
                let w = w.as_mut().unwrap();
                w.hide();
                self.state = State::Hidden;
            }
        );
    }

    pub fn launch_selected_application(&self) {
        let idx = match self.selected_search_idx {
            Some(0)|None => self.search_result_frames[0].get(),
            Some(idx) => self.search_result_frames[idx].get()
        };
        self.user_desktop_files.clone().unwrap()[*(idx.idx_in_xdg_entries_vector.borrow())].launch(None);
    }

    pub fn clear_search_buffer(&mut self) {
        self.text_input.clone().unwrap().set_text("");
    }

    pub fn scroll_search_results_down(&mut self) {
        const end_dx: usize = RESULT_ENTRY_COUNT - 1;
        match self.selected_search_idx {
            Some(end_dx) => {
                let next_search_result_idx = self.search_result_frames[
                    RESULT_ENTRY_COUNT - 1].get_idx_in_search_result_vector() + 1;
                let next_result_desktop_idx = search::get_xdg_index_from_last_search_result_idx(
                    &self.search_context.clone().unwrap().borrow(), next_search_result_idx);
                let next_result_desktop_idx = match next_result_desktop_idx {
                    Some(idx) => idx,
                    None => return
                };
                for i in 0..self.search_result_frames.len() - 1 {
                    let next_box = &self.search_result_frames[i+1];
                    let search_result_idx = next_box.get_idx_in_search_result_vector();
                    let desktop_idx = next_box.get_desktop_idx();
                    self.set_search_frame(desktop_idx, i, search_result_idx);
                }
                self.set_search_frame(
                    next_result_desktop_idx, 
                    RESULT_ENTRY_COUNT - 1, 
                    next_search_result_idx);
            },
            _ => ()
        }
    }


    pub fn set_search_frame(&mut self, desktop_idx: usize, container_idx: usize, search_result_idx: usize) {
        unsafe {
            let display_name = launcher.user_desktop_files.clone().unwrap()[desktop_idx].display_name.clone();
            let frame = &mut self.search_result_frames[container_idx];
            frame.set_label(Some(&display_name));
            frame.set_desktop_idx(desktop_idx);
            frame.set_idx_in_search_result_vector(search_result_idx);
            frame.set_focusable(true);
            frame.set_visible(true);
        }
    }
}


thread_local! {
    static WINDOW: RefCell<Option<gtk::ApplicationWindow>> = RefCell::new(None);
}

fn get_time_str() -> String {
    let date_time  =  chrono::offset::Local::now();
    let formatted = format!("{}", date_time.format("%a %d/%B %Y %H:%M:%S"));
    formatted
}

fn key_handler(ec: &gtk::EventControllerKey, 
        key: gdk::Key, _: u32, m: gdk::ModifierType) -> gtk::glib::Propagation {
    println!("key {} {}", key, m);
    unsafe {
        match key {
            gdk::Key::Escape => launcher.hide_window(),
            gdk::Key::Return => launcher.launch_selected_application(),
            gdk::Key::Down => launcher.scroll_search_results_down(),
            _ => ()
        };

        match launcher.selected_search_idx {
            Some(_) => (),
            None => return gtk::glib::Propagation::Proceed,
        };

        match key {
            gdk::Key::BackSpace => {
                launcher.focus_text_input();
                let input = launcher.text_input.clone().unwrap();
                let mut pos = (*launcher.search_context.clone().unwrap()).borrow().buf.len() as i32;
                input.delete_text(pos -1, pos);
                input.select_region(pos - 1, pos - 1);
                return gtk::glib::Propagation::Proceed;
            },
            _ => ()
        };

        let key_unicode = key.to_unicode();
        match key_unicode {
            Some(character) => {
                println!("--------");
                launcher.focus_text_input();
                let input = launcher.text_input.clone().unwrap();
                let mut pos = (*launcher.search_context.clone().unwrap()).borrow().buf.len() as i32;
                input.insert_text(
                   &character.to_string(), 
                   &mut pos);
                input.select_region(pos, pos);
               // let mut search_context = launcher.search_context.clone().unwrap();
               // let mut search_context = search_context.borrow_mut(); 
               // search::text_inserted(&mut search_context, pos, &character.to_string())
               // println!("{} {}", ec.forward(&(*input)), pos);
              //  ::propogate_event(input);
               // (*launcher.text_input.unwrap()).propagate_key_event();
            },
            None => ()
        };
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
                    launcher.clear_search_results();
                    launcher.clear_search_buffer();
                    launcher.focus_text_input();
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
                Err(..) => todo!("fix app crashing when unable to detect modifying css file"),
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
    let desktop_entries = Rc::new(search::get_xdg_desktop_entries());
    let search_context = buffer.context.clone();
    (*search_context).borrow_mut().set_desktop_files(desktop_entries.clone());
    launcher.search_context = Some(search_context);
    launcher.user_desktop_files = Some(desktop_entries);
    let search_entry = SearchEntry::new(buffer);
    launcher.input_buffer = Some(Rc::new(search_entry));
    let mut input_field = gtk::Entry::builder().xalign(0.5)
        .buffer(&*launcher.input_buffer.clone().unwrap()).build();
    input_field.set_halign(gtk::Align::Center);
    let context = input_field.style_context();
    context.add_class("input_field");
    w.init_layer_shell();

    w.set_layer(Layer::Overlay);
    // w.auto_exclusive_zone_enable(); for persistent topbar
    w.set_margin(Edge::Left, 800);
    w.set_margin(Edge::Right, 800);
    w.set_margin(Edge::Top, 400);

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

    let mut result_frames: Vec<SearchResultBox> = Vec::new();

    let mouse_controller = gtk::EventControllerMotion::new();
    let gcc_right_click = gtk::GestureClick::builder()
            .button(1).propagation_phase(PropagationPhase::Capture).build();
        gcc_right_click.connect_pressed(handle_mouse_click);
    mouse_controller.connect_motion(listbox_hover_handler);
    // mouse_controller.connect_stylus_only_notify(listbox_hover_handler_new);
    result_box.add_controller(mouse_controller);

    for i in 0..RESULT_ENTRY_COUNT {
        let frame = SearchResultBoxWidget::from(i);
        let frame = search_result_box_impl::SearchResultBox::new(frame);
        frame.set_focusable(true);
        frame.set_label(Some(""));
        frame.connect_has_focus_notify(|f| {
            launcher.selected_search_idx = Some(f.get().idx_in_container);
        });
        let context = frame.style_context();
        context.add_class("result_frame");
        result_frames.push(frame.into());
    }

    result_box.add_controller(gcc_right_click);
    for f in &result_frames  {
        result_box.append(f);
    }
    launcher.search_result_frames = result_frames;

    let ec = gtk::EventControllerKey::builder()
        .propagation_phase(PropagationPhase::Capture).build();
    ec.connect_key_pressed(key_handler);
    w.add_controller(ec);
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
    input_field.set_focusable(true);
    input_field.grab_focus_without_selecting();
    w.set_keyboard_mode(KeyboardMode::Exclusive);
    w.show();    
    launcher.text_input = Some(Rc::new(input_field.clone()));
    let input_field = &mut input_field;
    input_field.connect_has_focus_notify(|f| {
        launcher.selected_search_idx = None;
    });
    launcher.clear_search_results();
    WINDOW.replace(Some(w));

}

fn main()  -> gtk::glib::ExitCode {
    unsafe {
        launcher.state = State::Visible;
        let application = gtk::Application::new(
            Some("www.generic_launcher_example"), Default::default());
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