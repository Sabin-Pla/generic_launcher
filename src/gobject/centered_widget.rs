use gtk::subclass::prelude::{ObjectSubclass, ObjectImpl, WidgetImpl, LayoutManagerImpl, ObjectImplExt, ObjectSubclassExt};
use gtk::prelude::{LayoutManagerExt, WidgetExt};

mod inner {
    use super::*;

    pub struct CenteredWidget {
        pub icon: gtk::Image,
    }

    #[gtk::glib::object_subclass]
    impl ObjectSubclass for CenteredWidget {
        const NAME: &'static str = "CenteredWidget";
        type Type = super::CenteredWidget;
        type ParentType = gtk::Widget;

        fn new() -> Self {
            Self { icon: Default::default() }
        }
    }

    impl WidgetImpl for CenteredWidget {}

    impl ObjectImpl for CenteredWidget {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().set_layout_manager(Some(super::CenteredLayout::new()));
        }
    }

    #[derive(Default)]
    pub struct CenteredLayout {
        bin: gtk::BinLayout
    }

    
    #[gtk::glib::object_subclass]
    impl ObjectSubclass for CenteredLayout {
        const NAME: &'static str = "CenteredLayout";
        type Type = super::CenteredLayout;
        type ParentType = gtk::LayoutManager;
    }

    impl ObjectImpl for CenteredLayout {}

    impl LayoutManagerImpl for CenteredLayout {
        fn measure(
            &self,
            widget: &gtk::Widget,
            orientation: gtk::Orientation,
            for_size: i32,
        ) -> (i32, i32, i32, i32) {
            let parent = widget.parent().unwrap();
            let parent_allocation = parent.compute_bounds(&parent.parent().unwrap());
            println!("widget parent allocation (measure) {:?} {for_size}", &parent_allocation);
            self.bin.measure(widget, orientation, for_size)
        }

        fn allocate(&self, widget: &gtk::Widget, _width: i32, _height: i32, baseline: i32) {
            let parent = widget.parent().unwrap();
            let parent_allocation = parent.compute_bounds(&parent.parent().unwrap()).unwrap();
            let width = parent_allocation.width() as i32;
            let height = parent_allocation.height() as i32;
            self.bin.allocate(widget, width, height, baseline);
        }
    }
}

glib::wrapper! {
    pub struct CenteredWidget(ObjectSubclass<inner::CenteredWidget>)
    @extends gtk::Widget, gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

glib::wrapper! {
    pub struct CenteredLayout(ObjectSubclass<inner::CenteredLayout>)
    @extends gtk::LayoutManager;
}

use glib::prelude::IsA;
impl CenteredWidget {
    pub fn new(widget: &impl IsA<gtk::Widget>) -> Self {
        let obj = glib::Object::new::<Self>();
        widget.set_parent(&obj);
        widget.set_halign(gtk::Align::Center);
        obj
    }
}

impl CenteredLayout {
    pub fn new() -> Self {
        glib::Object::new::<Self>()
    }
}