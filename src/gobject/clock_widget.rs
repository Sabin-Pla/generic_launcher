
use gtk::glib::{self, Object};
use gtk::subclass::prelude::*;
use gtk::subclass::prelude::DerivedObjectProperties;
use gtk::prelude::WidgetExt;
use gtk::prelude::ObjectExt;
use gtk::Widget;
use glib::prelude::IsA;
use gtk::subclass::layout_manager::LayoutManagerImplExt;
use gtk::prelude::LayoutManagerExt;

mod inner {
    use super::*;

    pub struct ClockWidget(pub gtk::Label);

    #[gtk::glib::object_subclass]
    impl ObjectSubclass for ClockWidget {
        const NAME: &'static str = "ClockWidget";
        type Type = super::ClockWidget;
        type ParentType = gtk::Widget;

        fn new() -> Self {
            Self(gtk::Label::new(Some("TEST")))
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

    pub struct ClockLayout{
        bin: gtk::BinLayout
    }

    impl Default for ClockLayout {
        fn default() -> Self {
            Self {
                bin: gtk::BinLayout::new(),
            }
        }
    }

    #[gtk::glib::object_subclass]
    impl ObjectSubclass for ClockLayout {
        const NAME: &'static str = "ClockLayout";
        type Type = super::ClockLayout;
        type ParentType = gtk::LayoutManager;
    }

    impl ObjectImpl for ClockLayout {}

    impl LayoutManagerImpl for ClockLayout {
        fn measure(
            &self,
            widget: &Widget,
            orientation: gtk::Orientation,
            for_size: i32,
        ) -> (i32, i32, i32, i32) {
            self.bin.measure(widget, orientation, for_size)
        }

        fn allocate(&self, widget: &gtk::Widget, width: i32, height: i32, baseline: i32) {
            println!("ALLOCATE CALLED {width} {height}");
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
    pub fn new() -> Self {
        let obj = Object::new::<Self>();
        let clock_label = &inner::ClockWidget::from_obj(&obj).0; 
        clock_label.set_parent(&obj);
        setup_on_clock_tick(clock_label);
        obj.set_child_visible(true);
        obj
    }
}

impl ClockLayout {
    pub fn new() -> Self {
        let obj = Object::new::<Self>();
        obj
    }
}

fn set_clock_time(clock: &gtk::Label) {
    let date_time  =  chrono::offset::Local::now();
    let time_string = &format!("{}", date_time.format("%a %d/%B %Y %H:%M:%S"));
    clock.set_text(time_string);
}

fn calculate_clock_padding(clock: &gtk::Label) -> i32 {
    let width = clock.width();
    println!("calculate_clock_padding width {width}");
    let num_chars = clock.text().len();
    width + ((width / num_chars as i32) as f32 * 3.5) as i32
}

fn setup_on_clock_tick(clock_label: &gtk::Label) {
    let clock_label = clock_label.clone();
    let on_tick =  move || -> glib::ControlFlow {
        set_clock_time(&clock_label.clone());
        glib::ControlFlow::Continue
    };
    glib::timeout_add_seconds_local(1, on_tick); 
}
