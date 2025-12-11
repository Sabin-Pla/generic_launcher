use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::{EditableExt, EntryExt, FileExt, GridExt,  WidgetExt};

use super::State;
use crate::gobject::{SearchEntryBuffer, SearchResultContainer, SearchResultBox};
use crate::search;
use crate::search::SearchContext;
use crate::xdg_desktop_entry::XdgDesktopEntry;

use crate::WINDOW;
use crate::launcher::RESULT_ENTRY_COUNT;

pub struct Launcher {
    pub state: State,
    pub css_provider: Option<(std::sync::Arc<gio::File>, gtk::CssProvider)>,
    pub search_result_container: SearchResultContainer,
    pub selected_search_idx: Option<isize>,
    pub search_bar: Rc<gtk::Entry>,
    pub user_desktop_files: Option<Rc<Vec<XdgDesktopEntry>>>,
    pub search_context: SearchContext,
    pub input_buffer: Option<RefCell<SearchEntryBuffer>>,
    pub custom_launchers: Option<Rc<Vec<XdgDesktopEntry>>>,
    pub screenshot_button: Rc<gtk::Image>,
    pub hovered_idx: usize,
    pub current_monitor: Rc<RefCell<Option<(i32, i32)>>>,
    pub search_results_cache: search::SearchResult,
    hovering_suppressed: bool,
}

impl Launcher {
    pub fn uninitialized() -> Self {
        Launcher {
            state: State::NotStarted,
            css_provider: None,
            search_result_container: SearchResultContainer::new(),
            selected_search_idx: None,
            search_bar: Default::default(),
            user_desktop_files: None,
            search_context: SearchContext::default(),
            input_buffer: None,
            custom_launchers: None,
            screenshot_button: Default::default(),
            hovered_idx: 0,
            current_monitor: Rc::new(RefCell::new(None)),
            search_results_cache: Vec::new(),
            hovering_suppressed: false,
        }
    }

    pub fn disable_motion_events(&mut self) {
        self.hovering_suppressed = true;
    }

    pub fn enable_motion_events(&mut self) {
        self.hovering_suppressed = false;
    }

    pub fn launch_selected_application(&mut self) {
        let search_result_box = match self.selected_search_idx {
            Some(-1) => {
                self.custom_launchers.clone().unwrap()[0].launch(None);
                return;
            }
            Some(0) | None => self.search_result_container.index(0),
            Some(idx) => self.search_result_container.index(idx as usize),
        };
        self.user_desktop_files.clone().unwrap()[search_result_box.get().idx_in_xdg_entries_vector].launch(None);
    }

    pub fn set_search_result_box(
        &self,
        desktop_idx: usize,
        container_idx: usize,
        search_result_idx: usize,
    ) {
        let desktop_entry = &self.user_desktop_files.clone().unwrap()[desktop_idx];
        let display_name = desktop_entry.display_name.clone();
        let search_result_box = &mut self.search_result_container.index(container_idx);
        gtk::prelude::ButtonExt::set_label(search_result_box, &display_name);
        search_result_box.set_desktop_idx(desktop_idx);
        search_result_box.set_idx_in_search_result_vector(search_result_idx);
        search_result_box.set_focusable(true);
        search_result_box.set_visible(true);
        let app_info = desktop_entry.app_info.clone();
        /*
        if app_info.has_key("Icon") {
            let icon_name = app_info.locale_string("Icon").unwrap();
            let image = gtk::Image::from_icon_name(&icon_name);
            println!("icon name {} {}", icon_name, image.uses_fallback());
            let root = gtk::Grid::builder()
                .hexpand(true)
                .vexpand(true)
                .column_spacing(100)
                .build();
            root.attach(&image, 1, 1, 3, 20);
            result_box.set_icon(&icon_name);
        } 
        */
        let search_result_box = &mut self.search_result_container.index(container_idx);
    }

    pub fn reload_css(&mut self) {
        println!("reloading css...");
        match &self.css_provider {
            Some((file, provider)) => {
                provider.load_from_path(file.path().expect("invalid path for css provider"))
            }
            None => (),
        };
    }

    pub fn hide_search_results_container(&self) {
        self.search_result_container.hide();
    }

     pub fn show_search_results_container(&self) {
        self.search_result_container.show();
    }

    pub fn adjust_results_scrollbar(&self) {
        let selected_result_box = self.search_result_container.index(0);
        let search_result_count = self.search_results_cache.len();
        self.search_result_container.adjust_scrollbar(
            selected_result_box.get_idx_in_search_result_vector(), 
            search_result_count, 
            std::cmp::min(RESULT_ENTRY_COUNT, search_result_count));
    }

    pub fn set_search_results_cache(&mut self, search_results: search::SearchResult) {
        self.search_results_cache = search_results
    }
}

pub fn handle_enter_key(launcher_cell: Rc<RefCell<Launcher>>) {
    let mut launcher = launcher_cell.borrow_mut();
    if ! launcher.search_result_container.is_visible() {
        if launcher.search_bar.text() == "" {
            println!("Enter pressed with empty search bar - showing all apps");
            let search_results = search::refetch_results(&mut launcher.search_context, "\n".to_string());
            launcher.set_search_results_cache(search_results);
            launcher.show_search_results_container();
            search::display_search_results(&mut launcher, None);
        }
        return;
    }
    if let Some(_idx) = launcher.selected_search_idx {
        launcher.launch_selected_application();
        drop(launcher);
        hide_window(launcher_cell)
    } else {
        let search_result_box = launcher.search_result_container.index(0);
        drop(launcher);
        search_result_box.grab_focus();
    };
}

pub fn hide_window(launcher: Rc<RefCell<Launcher>>) {
    WINDOW.with(|application_window| {
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
            let next_page_top = launcher.search_result_container.index(1)
                .get_idx_in_search_result_vector();
            println!("next_page_top {} / {}", next_page_top,  launcher.search_results_cache.len());
            let end = launcher.search_results_cache.len();
            if end < RESULT_ENTRY_COUNT || next_page_top > end - RESULT_ENTRY_COUNT {
                return;
            }
            search::display_search_results(&mut launcher, Some(next_page_top));
        }
        _ => (),
    }
    launcher.adjust_results_scrollbar();
}

pub fn scroll_search_results_up(launcher: Rc<RefCell<Launcher>>) -> bool {
    let mut launcher = launcher.borrow_mut();
    const END_IDX: isize = (RESULT_ENTRY_COUNT - 1) as isize;
    match launcher.selected_search_idx {
        Some(0) => {
            let prev_search_result_idx = launcher.search_result_container.index(0)
                .get_idx_in_search_result_vector();
            if prev_search_result_idx == 0 {
                return false;
            }
            search::display_search_results(&mut launcher, Some(prev_search_result_idx - 1));
            launcher.adjust_results_scrollbar();
            return true;
        }
        _ => (),
    }
    false
}

pub fn focus_text_input(launcher: Rc<RefCell<Launcher>>) {
    let search_bar = &launcher.borrow().search_bar.clone();
    if !search_bar.has_focus() {
        search_bar.grab_focus_without_selecting();
    }
}

pub fn handle_result_box_hovered(launcher: Rc<RefCell<Launcher>>, hovered_idx: usize) {
    let mut launcher = launcher.borrow_mut();
    if launcher.hovering_suppressed {
        launcher.enable_motion_events();
        return;
    }
    launcher.hovered_idx = hovered_idx;
    launcher.selected_search_idx = Some(hovered_idx as isize);
    let search_result_box = launcher.search_result_container.index(hovered_idx).clone();
    drop(launcher);
    search_result_box.grab_focus();
}
