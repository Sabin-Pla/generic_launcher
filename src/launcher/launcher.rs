use crate::HashMap;

use gtk::prelude::*;

use crate::{Rc, RefCell};
use crate::gobject::{SearchResultBox, SearchEntryBuffer};
use crate::xdg_desktop_entry::XdgDesktopEntry;
use crate::search;
use crate::search::{SearchContext};
use super::State;

use crate::launcher::RESULT_ENTRY_COUNT;
use crate::WINDOW;

pub struct Launcher {
    pub state: State,
    pub css_provider: Option<(
        std::sync::Arc<gio::File>, 
        Rc<gtk::CssProvider>)>,
    pub search_result_frames: Vec<SearchResultBox>,
    pub selected_search_idx: Option<isize>,
    pub search_bar: Rc<gtk::Entry>,
    pub user_desktop_files: Option<Rc<Vec<XdgDesktopEntry>>>,
    pub search_context: SearchContext,
    pub input_buffer: Option<RefCell<SearchEntryBuffer>>,
    pub custom_launchers: Option<Rc<Vec<XdgDesktopEntry>>>,
    pub screenshot_button: Rc<gtk::Image>,
    pub hovered_idx: usize,
    pub current_monitor: Option<(i32, i32)>,
    hovering_suppressed: bool,
}


impl Launcher {
    pub fn uninitialized() -> Self {
        Launcher { 
            state: State::NotStarted, 
            css_provider: None,
            search_result_frames: vec!(),
            selected_search_idx: None,
            search_bar: Default::default(),
            user_desktop_files: None,
            search_context: SearchContext::default(),
            input_buffer: None,
            custom_launchers: None,
            screenshot_button: Default::default(),
            hovered_idx: 0,
            current_monitor: None,
            hovering_suppressed: false
        }
    }

    pub fn disable_motion_events(&mut self) {
        println!("disable_motion_events()");
        self.hovering_suppressed = true;
    }

    pub fn enable_motion_events(&mut self) {
        println!("enable_motion_events()");
        self.hovering_suppressed = false;
    }

    pub fn clear_search_results(&mut self) {
        for result_box in &self.search_result_frames {
            result_box.set_focusable(false);
            result_box.set_visible(false);
        }
    }

    pub fn launch_selected_application(&mut self) {
        println!("launch_selected_application()");
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
        self.search_bar.clone().set_text("");
    }

    pub fn set_search_frame(&mut self, desktop_idx: usize, container_idx: usize, search_result_idx: usize) {
        let desktop_entry = &self.user_desktop_files.clone().unwrap()[desktop_idx];
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
            // result_box.set_icon(&icon_name);
        }

    }

    pub fn reload_css(&mut self) {
        // todo!("Call gtk4::style_context_remove_provider_for_display");
        println!("reloading css...");
        match &self.css_provider {
            Some((file, provider)) => provider.load_from_path(
                file.path().expect("invalid path for css provider")),
            None => ()
        };
    }
}

pub fn handle_enter_key(launcher_cell: Rc<RefCell<Launcher>>) {
    let mut launcher = launcher_cell.borrow_mut();
    if let Some(_idx) = launcher.selected_search_idx {
        launcher.launch_selected_application();
        drop(launcher);
        hide_window(launcher_cell)
    } else {
        let result_box = launcher.search_result_frames[0].clone();
        drop(launcher);
        result_box.grab_focus();
    };
}


pub fn hide_window(launcher: Rc<RefCell<Launcher>>) {
    WINDOW.with( |application_window| {
        println!("Hiding window");
        let mut application_window = (*application_window).borrow_mut();
        let application_window = application_window.as_mut().unwrap();
        application_window.set_visible(false);
        let mut launcher = launcher.borrow_mut();
        launcher.state = State::Hidden;
    });
}

pub fn scroll_search_results_down(launcher: Rc<RefCell<Launcher>>) {
    let mut launcher = launcher.borrow_mut();
    const END_IDX: isize = (RESULT_ENTRY_COUNT - 1) as isize;
    match launcher.selected_search_idx {
        Some(END_IDX) => {
            let next_search_result_idx = launcher.search_result_frames[
                RESULT_ENTRY_COUNT - 1].get_idx_in_search_result_vector() + 1;
            let next_result_desktop_idx = search::get_xdg_index_from_last_search_result_idx(
                &launcher.search_context, next_search_result_idx);
            let next_result_desktop_idx = match next_result_desktop_idx {
                Some(idx) => idx,
                None => return
            };
            for i in 0..launcher.search_result_frames.len() - 1 {
                let next_box = &launcher.search_result_frames[i+1];
                let search_result_idx = next_box.get_idx_in_search_result_vector();
                let desktop_idx = next_box.get_desktop_idx();
                launcher.set_search_frame(desktop_idx, i, search_result_idx);
            }
            launcher.set_search_frame(
                next_result_desktop_idx, 
                RESULT_ENTRY_COUNT - 1, 
                next_search_result_idx);
        },
        _ => ()
    }
}

pub fn focus_text_input(launcher: Rc<RefCell<Launcher>>) -> bool {
    let mut launcher = launcher.borrow_mut();
    let search_bar = launcher.search_bar.clone();
    drop(launcher);
    if !search_bar.has_focus() {
        search_bar.grab_focus_without_selecting();
        return true;
    }
    false
}

pub fn handle_result_box_hovered(launcher: Rc<RefCell<Launcher>>, hovered_idx: usize) {
    let mut launcher = launcher.borrow_mut();
    if launcher.hovering_suppressed {
        println!("Hovering is suppressed");
        launcher.enable_motion_events();
        return;
    }
    launcher.hovered_idx = hovered_idx;
    println!("launcher handle hover: {hovered_idx}");
    launcher.selected_search_idx = Some(hovered_idx as isize);
    let result_box = launcher.search_result_frames[hovered_idx].clone();
    drop(launcher);
    println!("refocusing result box {hovered_idx}");
    result_box.grab_focus();
}