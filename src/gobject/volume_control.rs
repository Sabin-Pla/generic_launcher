use std::cell::RefCell;
use std::rc::Rc;

use gtk::glib::{Object};
use gtk::prelude::{ButtonExt, Cast, LayoutManagerExt, WidgetExt};
use gtk::subclass::prelude::*;

mod inner {
    use super::*;

    #[derive(Default)]
    pub struct VolumeControl{}

    #[gtk::glib::object_subclass]
    impl ObjectSubclass for VolumeControl {
        const NAME: &'static str = "VolumeControl";
        type Type = super::VolumeControl;
        type ParentType = gtk::Button;
    }

    impl ObjectImpl for VolumeControl {}

    impl WidgetImpl for VolumeControl {}

    impl ButtonImpl for VolumeControl {}

}

glib::wrapper! {
    pub struct VolumeControl(ObjectSubclass<inner::VolumeControl>)
    @extends gtk::Button, gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget, gtk::Widget;
}

impl VolumeControl {
    pub fn new(icon_theme: &gtk::IconTheme) -> Self {
        let obj = Object::new::<Self>();
        let volume_paintable = icon_theme.lookup_icon(
            "audio-volume-high-symbolic",
            &[],
            32,
            1,
            gtk::TextDirection::None,
            gtk::IconLookupFlags::PRELOAD,
        );
        let mut volume_icon = gtk::Image::from_paintable(Some(&volume_paintable));
        volume_icon.set_icon_size(gtk::IconSize::Large);

        obj.set_child(Some(&volume_icon));
        obj.set_has_frame(false);
        obj
    }
}

