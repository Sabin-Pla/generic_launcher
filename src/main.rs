
use std::path::{Path, PathBuf};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::os::raw::c_char;
use std::fs::File;
use std::collections::HashMap;
use std::time::Duration;

use libc::{c_void, mkfifo, O_RDONLY, O_WRONLY};

use gio::prelude::*;
use gtk::prelude::*;
use glib::StrV;
use gtk4_layer_shell::{KeyboardMode, Edge, Layer, LayerShell};
use gtk::PropagationPhase;
use gtk::prelude::ApplicationExtManual;

mod search;
mod search_buffer_imp;
mod xdg_desktop_entry;
use xdg_desktop_entry::XdgDesktopEntry;
use search_buffer_imp::SearchEntry;
use search::SearchContext;

mod search_result_box_impl;
use search_result_box_impl::{SearchResultBoxWidget, SearchResultBox};

use inotify::{
    Inotify,
    WatchMask,
};

pub const RESULT_ENTRY_COUNT: usize = 6;


pub struct Launcher {
    state: State,
    done_init: bool,
    css_provider: Option<(
        Box<PathBuf>, 
        Arc<File>, 
        Rc<gtk::CssProvider>)>,
    fifo_path: [i8; 2000],
    search_result_frames: Vec<SearchResultBox>,
    selected_search_idx: Option<isize>,
    text_input: Option<Rc<gtk::Entry>>,
    user_desktop_files: Option<Rc<Vec<XdgDesktopEntry>>>,
    search_context: Option<Rc<RefCell<SearchContext>>>,
    input_buffer: Option<Rc<SearchEntry>>,
    custom_launchers: Option<Rc<Vec<XdgDesktopEntry>>>,
    screenshot_button: Option<Rc<gtk::Image>>,
    hovered_idx: usize,
    clock: Option<Rc<std::cell::RefCell<gtk::Label>>>,
    clock_sizes: Option<HashMap<(i32, i32), i32>>,
    current_monitor: Option<(i32, i32)>,
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
    input_buffer: None,
    custom_launchers: None,
    screenshot_button: None,
    hovered_idx: 0,
    clock: None,
    clock_sizes: None,
    current_monitor: None,
}; 

impl Launcher {
    pub fn clear_search_results(&mut self) {
        for result_box in &self.search_result_frames {
            //frame.set_label(Some(""));
            result_box.set_focusable(false);
            result_box.set_visible(false);
        }
    }

    pub fn focus_text_input(&mut self) {
        self.text_input.clone().unwrap().grab_focus();
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

    pub fn handle_enter_key(&mut self) {
        if let Some(_idx) = self.selected_search_idx {
            self.launch_selected_application();
            self.hide_window();
        } else {
            self.search_result_frames[0].grab_focus();
        };
    }

    pub fn launch_selected_application(&mut self) {
        let idx = match self.selected_search_idx {
            Some(-1) => {
                self.custom_launchers.clone().unwrap()[0].launch(None);
                return;
            }, 
            Some(0)|None => self.search_result_frames[0].get(),
            Some(idx) => self.search_result_frames[idx as usize].get()
        };
        self.user_desktop_files.clone().unwrap()[idx.idx_in_xdg_entries_vector].launch(None);
    }

    pub fn clear_search_buffer(&mut self) {
        self.text_input.clone().unwrap().set_text("");
    }

    pub fn scroll_search_results_down(&mut self) {
        self.deselect_text();
        const END_IDX: isize = (RESULT_ENTRY_COUNT - 1) as isize;
        match self.selected_search_idx {
            Some(END_IDX) => {
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
            let desktop_entry = &launcher.user_desktop_files.clone().unwrap()[desktop_idx];
            let display_name = desktop_entry.display_name.clone();
            let result_box = &mut self.search_result_frames[container_idx];
            gtk::prelude::ButtonExt::set_label(result_box, &display_name);
            result_box.set_desktop_idx(desktop_idx);
            result_box.set_idx_in_search_result_vector(search_result_idx);
            result_box.set_focusable(true);
            result_box.set_visible(true);
            let app_info = desktop_entry.app_info.clone();
            if app_info.has_key("Icon") {
                let icon_name = app_info.locale_string("Icon").unwrap();
                let image = gtk::Image::from_icon_name(&icon_name);
                println!("icon name {} {}", icon_name, image.uses_fallback());
                let root = gtk::Grid::builder().hexpand(true).vexpand(true).column_spacing(100).build();
                root.attach(&image, 1, 1, 3, 20);
                //result_box.set_icon(&icon_name);
            }
        }
    }

    pub fn select_screenshot_button(&mut self) {
        let button = self.screenshot_button.clone().unwrap();
        button.grab_focus();
    }

    pub fn handle_hovered(&mut self, hovered_idx: usize) {
        unsafe {
            launcher.hovered_idx = hovered_idx;
            self.search_result_frames[hovered_idx].grab_focus();
        }
    }

    pub fn handle_result_click(&mut self, clicked_idx: usize) {
        if self.search_result_frames[clicked_idx].has_focus() {
            self.launch_selected_application();
            self.hide_window();
        } else {
            self.search_result_frames[clicked_idx].grab_focus();
        }
    }

    pub fn deselect_text(&mut self) {
        unsafe {
            let input = self.text_input.clone().unwrap();
            let pos = (*launcher.search_context.clone().unwrap()).borrow().buf.len() as i32;
            input.select_region(pos - 1, pos - 1);
        }
    }
}


thread_local! {
    static WINDOW: RefCell<Option<gtk::ApplicationWindow>> = RefCell::new(None);
}

fn screenshot_enter_handler(_ec: &gtk::EventControllerMotion, _: f64, _: f64) {
    unsafe { launcher.select_screenshot_button(); }
}

fn screenshot_leave_handler(_ec: &gtk::EventControllerMotion) {
    unsafe { launcher.focus_text_input(); }
}

fn screenshot_click_handler(_gc: &gtk::GestureClick, _: i32, _: f64, _: f64) {
    unsafe {
        launcher.select_screenshot_button();
        launcher.launch_selected_application();
        launcher.hide_window();
    }
}

fn get_time_str() -> String {
    let date_time  =  chrono::offset::Local::now();
    format!("{}", date_time.format("%a %d/%B %Y %H:%M:%S"))
} 


fn key_handler(_ec: &gtk::EventControllerKey, 
        key: gdk::Key, _: u32, m: gdk::ModifierType) -> gtk::glib::Propagation {
    println!("key {} {:?}", key, m);
    unsafe {
        match key {
            gdk::Key::Escape => launcher.hide_window(),
            gdk::Key::Return => launcher.handle_enter_key(),
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
                let pos = (*launcher.search_context.clone().unwrap()).borrow().buf.len() as i32;
                input.delete_text(pos -1, pos);
                input.select_region(pos - 1, pos - 1);
                return gtk::glib::Propagation::Proceed;
            },
            _ => ()
        };

        let key_unicode = key.to_unicode();
        match key_unicode {
            Some('\r') => (),
            Some(character) => {
                launcher.focus_text_input();
                let input = launcher.text_input.clone().unwrap();
                let mut pos = (*launcher.search_context.clone().unwrap()).borrow().buf.len() as i32;
                input.insert_text(
                   &character.to_string(), 
                   &mut pos);
                input.select_region(pos, pos);
            },
            None => ()
        };
    }


    gtk::glib::Propagation::Proceed
}

unsafe fn activate(_application: &gtk::Application) {
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
                    let clock = unsafe {
                        launcher.clock.clone().expect("Clock not initialized")
                    };
                    let clock = clock.borrow();
                    let display = clock.display();
                    let surface = w.surface().unwrap();
                    let display = display.monitor_at_surface(&surface);
                    let rect =  display.unwrap().geometry();
                    let (width, height) = (rect.width(), rect.height());
                    println!("monitor: {width} {height}");
                    launcher.current_monitor = Some((width, height));
                    set_clock_size(w, &mut launcher.clock_sizes);
                    launcher.clear_search_results();
                    launcher.clear_search_buffer();
                    launcher.focus_text_input();
  					launcher.state = State::Visible;
  				}
  			}
		});
	} else {
        println!("not yet initialized...");
    }
	launcher.done_init = true;
}

fn reload_css() {
    println!("reloading css...");
    unsafe {
        match &launcher.css_provider {
            Some((path, _file, provider)) => 
                provider.load_from_path((*path).as_path()),
            None => ()
        };
        let mut clock_sizes = unsafe { &mut launcher.clock_sizes };
        *clock_sizes = Some(HashMap::new());
    }
}

fn set_clock_time(time: &String, clock: &gtk::Label) {
    clock.set_text(&time);
}

fn show_clock() {
    let clock = unsafe {
        launcher.clock.clone().expect("Clock not initialized")
    };
    let mut clock_sizes = unsafe { &mut launcher.clock_sizes };
    let current_monitor = unsafe { launcher.current_monitor.expect("current monitor not set") }; 
    let clock_sizes = clock_sizes.as_mut();
    let clock_sizes = clock_sizes.expect("clock_sizes not defined");
    let clock = clock.borrow();
    let padded = calculate_clock_padding(&clock);
    let clock_width = clock_sizes.entry(current_monitor).or_insert(padded);
    if *clock_width == 0 {
        *clock_width = padded;
    }
    println!("Showing clock and setting width to {padded}");
    println!("{:?}", &clock_sizes);
    clock.set_opacity(1.0);
    clock.set_size_request(padded, 0);
}

fn calculate_clock_padding(clock: &gtk::Label) -> i32 {
    let width = clock.width();
    let num_chars = clock.text().len();
    width + ((width / num_chars as i32) as f32 * 3.5) as i32
}

fn set_clock_size(
        w: &gtk::ApplicationWindow, 
        clock_sizes: &mut Option<HashMap<(i32, i32), i32>>) {
    let clock = unsafe {
        launcher.clock.clone().expect("Clock not initialized")
    };
    let clock = clock.borrow();
  
    let display = clock.display().monitor_at_surface(&w.surface().unwrap());
    let rect =  display.unwrap().geometry();
    let (w, h) = (rect.width(), rect.height());
    // width might change substantially when app is opened on different monitor
    // so save a different padded clock width for each monitor and use that
    let clock_sizes = clock_sizes.as_mut();
    let clock_sizes = clock_sizes.expect("clock_sizes not defined");
    let uninitialized = clock_sizes.keys().len() == 0;

    let pad_clock = || {
        if uninitialized || clock.width() == 0 {
            clock.set_visible(false);
            clock.set_size_request(0, 0);
            clock.set_visible(true);
            println!("clock has 0 width {}", clock.width());
            // width may be zero in the event that widget hasn't loaded yet.
            // hide the clock while it loads.
            clock.set_opacity(0.0);
            glib::timeout_add_local_once(Duration::from_millis(20), show_clock);
        } 
        calculate_clock_padding(&clock)
    };
    let provider = gtk::CssProvider::new();
    // todo: make this add half the added width to padding-left. for now we'll just use 20px cause it looks good enough
    provider.load_from_string(".clock { padding-left: 20px }");
    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + 1);

    let clock_width = clock_sizes.entry((w, h)).or_insert_with(pad_clock);
    clock.set_size_request(*clock_width, 40);
}

unsafe fn startup(application: &gtk::Application) {
    println!("Starting up...");
    let mut screenshots_path = glib::home_dir();
    screenshots_path.push("Pictures");
    screenshots_path.push("Screenshots");
    let _ = std::fs::create_dir(screenshots_path);

    let mut css_path = glib::user_config_dir();
    css_path.push("generic_launcher/launcher.css");
    let css_file = match File::open(css_path.clone()) {
        Ok(f) => f,
        Err(..) => {
            let mut cwd = std::env::current_dir().expect("Error accessing CWD");
            cwd.push("launcher.css");
            let mut parent_dir = glib::user_config_dir();
            parent_dir.push("generic_launcher");
            let _ = std::fs::create_dir(parent_dir);
            let _ = std::os::unix::fs::symlink(cwd, css_path.clone());
            File::open(css_path.clone()).unwrap()
        }
    };

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
        Some((path, _file, provider)) =>  {

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
                                                if !matches!(event.mask, 
                                                    inotify::EventMask::CLOSE_NOWRITE) {
                                                    libc::write(fd, 
                                                        buffer.as_ptr() as *mut c_void, 1);
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
            
            let (fd, buffer) = open_pipe(O_RDONLY);
            glib::source::unix_fd_add_local(
                fd, 
                glib::IOCondition::IN, move |_, _d| {
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

    let buffer = search_buffer_imp::SearchEntryBuffer::new();
    let xdg_desktop_entries = search::get_xdg_desktop_entries();
    let desktop_entries = Rc::new(xdg_desktop_entries.0);
    let custom_launchers = Rc::new(xdg_desktop_entries.1);
    let search_context = buffer.context.clone();
    (*search_context).borrow_mut().set_desktop_files(desktop_entries.clone());
    launcher.search_context = Some(search_context);
    launcher.user_desktop_files = Some(desktop_entries.clone());
    launcher.custom_launchers = Some(custom_launchers);
    


   let ec = gtk::EventControllerKey::builder()
        .propagation_phase(PropagationPhase::Capture).build();
    ec.connect_key_pressed(key_handler);
    w.add_controller(ec);
    
    let ec = gtk::EventControllerKey::builder()
        .name("im_controller")
        .propagation_phase(PropagationPhase::Capture).build();
    let search_entry = SearchEntry::new(buffer);
    launcher.input_buffer = Some(Rc::new(search_entry));
    let im_context = search_buffer_imp::SearchEntryIMContext::new();

    let im_simple = gtk::IMContextSimple::new();
    ec.set_im_context(Some(&im_context));

    let mut input_field = gtk::Entry::builder().xalign(0.5)
        .buffer(&*launcher.input_buffer.clone().unwrap()).build();
    input_field.set_halign(gtk::Align::Center);
    let test= |x: &search_buffer_imp::SearchEntryIMContext| {
        println!("--------------");
        println!("--------------");
        println!("--------------");
        println!("--------------");
        todo!();
    };
   // ec.connect_im_update(test);
    im_context.connect_preedit_changed(test);
    input_field.add_controller(ec);
    im_context.set_use_preedit(true);
    let context = input_field.style_context();
    context.add_class("input-field");
    //todo!("{:?}", input_field.im_module());

    w.init_layer_shell();

    // w.auto_exclusive_zone_enable(); for persistent topbar
    w.set_layer(Layer::Overlay);
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

    for i in 0..RESULT_ENTRY_COUNT {
        let result_box = SearchResultBoxWidget::from(i);
        let result_box = search_result_box_impl::SearchResultBox::new(result_box);
        result_box.set_focusable(true);
        result_box.set_focus_on_click(true);
        gtk::prelude::ButtonExt::set_label(&result_box, &"");
        let gesture_click = gtk::GestureClick::builder()
            .propagation_phase(PropagationPhase::Capture).build();
        let ecm = gtk::EventControllerMotion::builder()
            .propagation_phase(PropagationPhase::Capture).build();
        gesture_click.connect_pressed(move |_, _, _, _| {
            launcher.handle_result_click(i)
        });
        ecm.connect_enter(move |_, _, _| { launcher.handle_hovered(i) });
        result_box.add_controller(gesture_click);
        result_box.add_controller(ecm);
        result_box.connect_has_focus_notify(|f| {
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
    launcher.search_result_frames = result_frames;

    let clock_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let clock = gtk::Label::default();
    // let clock_seconds = gtk::Label::default();
    clock_box.append(&clock);
    let context = clock.style_context();
    context.add_class("clock");
    clock.set_xalign(0.0);
    set_clock_time(&get_time_str(), &clock);

    let context = root.style_context();

    context.add_class("root");
    let topbar = gtk::CenterBox::builder()
        .orientation(gtk::Orientation::Horizontal)
        .build();
    topbar.set_center_widget(Some(&clock_box));
    
    let mut cwd = std::env::current_dir().expect("Error accessing CWD");
    cwd.push("assets");
    let resource_path = cwd.into_os_string().into_string().unwrap();
    let mut search_paths = StrV::new();
    search_paths.push(resource_path.clone().into());
    let icon_theme = gtk::IconTheme::builder()
        .theme_name("Adwaita")
        .build();
    
    println!("resource_path {resource_path}");
    icon_theme.set_resource_path(&[&resource_path]);
    icon_theme.set_search_path(&[Path::new(&resource_path)]);
    println!("{:?}", icon_theme.search_path());
    let screenshot_paintable = icon_theme.lookup_icon(
        "adwaita-applets-screenshooter-symbolic", &[], 
        32, 1, gtk::TextDirection::None, gtk::IconLookupFlags::PRELOAD);
    let screenshot_icon = gtk::Image::from_paintable(Some(&screenshot_paintable));
    screenshot_icon.set_icon_size(gtk::IconSize::Large);
    screenshot_icon.set_focusable(true);
    screenshot_icon.connect_has_focus_notify(|_f| {
        launcher.selected_search_idx = Some(-1);
    });

    let ecm = gtk::EventControllerMotion::builder()
        .propagation_phase(PropagationPhase::Capture).build();
    ecm.connect_enter(screenshot_enter_handler);
    ecm.connect_leave(screenshot_leave_handler);
    screenshot_icon.add_controller(ecm);
    let context = screenshot_icon.style_context();
    context.add_class("screenshot-button");
    launcher.screenshot_button = Some(Rc::new(screenshot_icon.clone()));

    let gesture_click = gtk::GestureClick::new();
    gesture_click.connect_pressed(screenshot_click_handler);
    screenshot_icon.add_controller(gesture_click);

    topbar.set_end_widget(Some(&screenshot_icon));

    input_field.connect_has_focus_notify(|_f| {
        launcher.selected_search_idx = None;
    });
    root.append(&topbar);
    let clock = Rc::new(RefCell::new(clock));
    launcher.clock = Some(clock.clone());
    launcher.clock_sizes = Some(HashMap::new());
    let tick = move || { 
        let clock = clock.borrow();
        set_clock_time(&get_time_str(), &clock);
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
    launcher.text_input = Some(Rc::new(input_field.clone()));
    let input_field = &mut input_field;
    input_field.set_placeholder_text(Some("Applications"));
    input_field.connect_has_focus_notify(|_f| {
        launcher.selected_search_idx = None;
    });
    input_field.set_has_frame(true);
    launcher.clear_search_results();
    WINDOW.replace(Some(w));

}


fn main()  -> gtk::glib::ExitCode {
    unsafe {
        launcher.state = State::Hidden;
        let application = gtk::Application::new(
            Some("www.generic_launcher_example"), Default::default());
        application.set_accels_for_action("win.close", &["<Ctrl>C"]);
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