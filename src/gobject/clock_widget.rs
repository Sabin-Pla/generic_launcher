use std::cell::RefCell;
use std::collections::{HashMap, hash_map};
use std::rc::Rc;

use gtk::glib::{Object};
use gtk::prelude::{Cast, LayoutManagerExt, WidgetExt};
use gtk::subclass::prelude::*;

mod inner {
    use super::*;

    pub struct ClockWidget(pub gtk::Label);

    #[gtk::glib::object_subclass]
    impl ObjectSubclass for ClockWidget {
        const NAME: &'static str = "ClockWidget";
        type Type = super::ClockWidget;
        type ParentType = gtk::Widget;

        fn new() -> Self {
            let clock_label = gtk::Label::new(Some(&get_time_str()));
            clock_label.set_halign(gtk::Align::Start);
            Self(clock_label)
        }
    }

    impl ObjectImpl for ClockWidget {
        fn constructed(&self) {
            self.parent_constructed();

            // Give this widget a layout manager
            let obj = self.obj();
            obj.set_layout_manager(Some(super::ClockLayout::new()));
        }
    }

    impl WidgetImpl for ClockWidget {}

    pub struct ClockLayout {
        bin: gtk::BinLayout,
        pub current_monitor: RefCell<Rc<RefCell<Option<(i32, i32)>>>>,
        padding_size_map: RefCell<HashMap<(i32, i32), (i32, i32)>>,
        css_provider: Option<gtk::CssProvider>,
    }

    impl Default for ClockLayout {
        fn default() -> Self {
            Self {
                bin: gtk::BinLayout::new(),
                current_monitor: Default::default(),
                padding_size_map: Default::default(),
                css_provider: Default::default(),
            }
        }
    }

    #[gtk::glib::object_subclass]
    impl ObjectSubclass for ClockLayout {
        const NAME: &'static str = "ClockLayout";
        type Type = super::ClockLayout;
        type ParentType = gtk::LayoutManager;
    }

    impl ClockLayout {
        fn check_padding_map(
            padding_size_map: &mut HashMap<(i32, i32), (i32, i32)>,
            current_dimensions: (i32, i32),
            width: i32,
            pango_font_size: i32,
            num_chars: i32
        ) -> Option<i32> {
            match Self::get_entry(padding_size_map, current_dimensions) {
                hash_map::Entry::Occupied(entry) => {
                    let val = entry.get().1;
                    if pango_font_size != val {
                        println!("Actual clock text size {pango_font_size} | expected {val}");
                        println!("(css-reload?) re-sizing clock.");
                        drop(entry);
                        padding_size_map.clear();
                        return Self::check_padding_map(padding_size_map, current_dimensions, width, pango_font_size, num_chars);
                    }
                    None
                }
                hash_map::Entry::Vacant(entry) => {
                    let padding = ((width / num_chars) as f32 * 2.3) as i32;
                    entry.insert((padding, pango_font_size));
                    Some(padding)
                },
            }
        }

        fn get_entry<'a>(
            padding_size_map: &'a mut HashMap<(i32, i32), (i32, i32)>,
            current_dimensions: (i32, i32),
        ) -> hash_map::Entry<'a, (i32, i32), (i32, i32)> {
            padding_size_map.entry(current_dimensions)
        }
    }

    impl ObjectImpl for ClockLayout {}

    impl LayoutManagerImpl for ClockLayout {
        fn measure(
            &self,
            widget: &gtk::Widget,
            orientation: gtk::Orientation,
            for_size: i32,
        ) -> (i32, i32, i32, i32) {
            self.bin.measure(widget, orientation, for_size)
        }

        fn allocate(&self, widget: &gtk::Widget, width: i32, height: i32, baseline: i32) {
            use std::borrow::BorrowMut;
            let current_monitor = self.current_monitor.borrow();
            let current_monitor = current_monitor.borrow();
            let current_dimensions =
                current_monitor.expect("ClockLayout could not determine monitor dimensions");

            let clock_widget = widget
                .clone()
                .downcast::<super::ClockWidget>()
                .expect("ClockLayout allocate() called for non-clock widget");
            let clock_widget = inner::ClockWidget::from_obj(&clock_widget);
            let clock_label = clock_widget.0.clone();
            let pango_font_size = clock_label.pango_context().font_description().unwrap().size(); 
            let num_chars = clock_label.text().len() as i32;
            let text_width = clock_label.layout().extents().1.width() / gtk::pango::SCALE;

            let padding = match Self::check_padding_map(
                &mut *self.padding_size_map.borrow_mut(),
                current_dimensions,
                text_width,
                pango_font_size,
                num_chars
            ) {
                Some(padding) => {
                    widget.set_size_request(2 * padding + text_width, height);
                    padding
                },
                None => {
                    self.bin.allocate(widget, width, height, baseline);
                    return
                },
            };

            match &self.css_provider {
                Some(css_provider) => gtk::style_context_remove_provider_for_display(&widget.display(), css_provider),
                None => {
                    let provider = gtk::CssProvider::new();
                    provider.load_from_string(&format!(".clock {{ padding-left: {padding}px; padding-right: 0px; }}"));
                    gtk::style_context_add_provider_for_display(
                        &widget.display(),
                        &provider,
                        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + 1,
                    );
                }
            };
            self.bin.allocate(widget, width, height, baseline);
        }
    }
}

glib::wrapper! {
    pub struct ClockWidget(ObjectSubclass<inner::ClockWidget>)
    @extends gtk::Widget, gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

glib::wrapper! {
    pub struct ClockLayout(ObjectSubclass<inner::ClockLayout>)
    @extends gtk::LayoutManager;
}

impl ClockWidget {
    pub fn new(monitor_cell: Rc<RefCell<Option<(i32, i32)>>>) -> Self {
        let obj = Object::new::<Self>();
        let clock_label = &inner::ClockWidget::from_obj(&obj).0;
        let mut layout_manager = obj
            .layout_manager()
            .expect("ClockLayout not created for ClockWidget")
            .downcast::<ClockLayout>()
            .expect("ClockLayout expected, got invalid LayoutManager class");

        layout_manager.set_monitor_cell(monitor_cell);
        clock_label.set_parent(&obj);
        setup_on_clock_tick(clock_label);
        obj.set_child_visible(true);
        obj.add_css_class("clock");
        obj
    }
}

impl ClockLayout {
    pub fn new() -> Self {
        glib::Object::new::<Self>()
    }

    pub fn set_monitor_cell(&mut self, monitor_cell: Rc<RefCell<Option<(i32, i32)>>>) {
        println!("set mon cell{:?}", &monitor_cell);
        use std::borrow::BorrowMut;
        let mut inner = inner::ClockLayout::from_obj(self);
        let mut inner_cell = inner.borrow_mut().current_monitor.borrow_mut();
        *inner_cell = monitor_cell;
    }
}

fn get_time_str() -> String {
    let date_time = chrono::offset::Local::now();
    format!("{}", date_time.format("%a %d/%B %Y %H:%M:%S"))
}

fn set_clock_time(clock: &gtk::Label) {
    clock.set_text(&get_time_str());
}

fn setup_on_clock_tick(clock_label: &gtk::Label) {
    let clock_label = clock_label.clone();
    let on_tick = move || -> glib::ControlFlow {
        set_clock_time(&clock_label.clone());
        glib::ControlFlow::Continue
    };
    glib::timeout_add_seconds_local(1, on_tick);
}
