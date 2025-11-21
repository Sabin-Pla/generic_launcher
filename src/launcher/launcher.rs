use crate::HashMap;

use gtk::prelude::*;

use crate::{Arc, Rc, RefCell};
use crate::gobject::{SearchResultBox, SearchEntry, SearchEntryBuffer};
use crate::xdg_desktop_entry::XdgDesktopEntry;
use crate::search;
use crate::search::{SearchContext};
use super::State;
use super::clock;

use crate::launcher::RESULT_ENTRY_COUNT;
use crate::WINDOW;

pub struct Launcher {
    pub state: State,
    pub done_init: bool,
    pub css_provider: Option<(
        Arc<gio::File>, 
        Rc<gtk::CssProvider>)>,
    pub fifo_path: [i8; 2000],
    pub search_result_frames: Vec<SearchResultBox>,
    pub selected_search_idx: Option<isize>,
    pub text_input: Option<Rc<gtk::Entry>>,
    pub user_desktop_files: Option<Rc<Vec<XdgDesktopEntry>>>,
    pub search_context: Option<Rc<RefCell<SearchContext>>>,
    pub input_buffer: Option<Rc<SearchEntry>>,
    pub custom_launchers: Option<Rc<Vec<XdgDesktopEntry>>>,
    pub screenshot_button: Option<Rc<gtk::Image>>,
    pub hovered_idx: usize,
    pub clock: Option<Rc<std::cell::RefCell<gtk::Label>>>,
    pub clock_sizes: Option<HashMap<(i32, i32), i32>>,
    pub current_monitor: Option<(i32, i32)>,
}


impl Launcher {
    pub fn clear_search_results(&mut self) {
        for result_box in &self.search_result_frames {
            result_box.set_focusable(false);
            result_box.set_visible(false);
        }
    }

    pub fn focus_text_input(&mut self) {
        self.text_input.clone().unwrap().grab_focus();
    }

    pub fn hide_window(&mut self) {
        WINDOW.with( |application_window| {
                let mut application_window = (*application_window).borrow_mut();
                let application_window = application_window.as_mut().unwrap();
                application_window.set_visible(false);
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
            //result_box.set_icon(&icon_name);
        }

    }

    pub fn select_screenshot_button(&mut self) {
        let button = self.screenshot_button.clone().unwrap();
        button.grab_focus();
    }

    pub fn handle_hovered(&mut self, hovered_idx: usize) {
        self.hovered_idx = hovered_idx;
        self.search_result_frames[hovered_idx].grab_focus();
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
        let input = self.text_input.clone().unwrap();
        let pos = (*self.search_context.clone().unwrap()).borrow().buf.len() as i32;
        input.select_region(pos - 1, pos - 1);
    }

    pub fn reload_css(&mut self) {
        // todo!("Call gtk4::style_context_remove_provider_for_display");
        println!("reloading css...");
        match &self.css_provider {
            Some((file, provider)) => provider.load_from_path(
                file.path().expect("invalid path for css provider")),
            None => ()
        };
        let clock_sizes = &mut self.clock_sizes;
        *clock_sizes = Some(HashMap::new());
    }

    pub fn set_clock_size(&mut self, application_window: &gtk::ApplicationWindow) {
        let clock = self.clock.clone().expect("cannot set uninitialized clock");
        clock::set_clock_size(application_window, &mut self.clock_sizes, clock);
    }
}
