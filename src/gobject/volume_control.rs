use std::cell::RefCell;
use std::rc::Rc;

use gtk::glib::{Object};
use gtk::prelude::{Cast, LayoutManagerExt, WidgetExt, BoxExt, PopoverExt};
use gtk::subclass::prelude::*;
use gdk::Rectangle;
use gtk4_layer_shell::LayerShell;

use crate::volume_mixer;

mod inner {
    use super::*;

    #[derive(Default)]
    pub struct VolumeControl { 
        pub popover: RefCell<Option<gtk::Popover>>,
    }

    #[gtk::glib::object_subclass]
    impl ObjectSubclass for VolumeControl {
        const NAME: &'static str = "VolumeControl";
        type Type = super::VolumeControl;
        type ParentType = gtk::Widget;
    }

    impl ObjectImpl for VolumeControl {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().set_layout_manager(Some(gtk::BinLayout::new()));
        }
    }

    impl WidgetImpl for VolumeControl {}
}

glib::wrapper! {
    pub struct VolumeControl(ObjectSubclass<inner::VolumeControl>)
    @extends gtk::Widget, gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl VolumeControl {
    pub fn new(
            icon_theme: &gtk::IconTheme, 
            application_window: &gtk::ApplicationWindow, 
            focus_on_hide: &impl gdk::prelude::IsA<gtk::Widget>) -> Self {
        let obj = Object::new::<Self>();
        let volume_control = &inner::VolumeControl::from_obj(&obj);
        let volume_paintable = icon_theme.lookup_icon(
            "audio-volume-high-symbolic",
            &[],
            32,
            1,
            gtk::TextDirection::None,
            gtk::IconLookupFlags::PRELOAD,
        );
        let volume_icon = gtk::Image::from_paintable(Some(&volume_paintable));
        volume_icon.set_icon_size(gtk::IconSize::Large);

        let popover = gtk::Popover::new();

        volume_icon.set_parent(&obj);
        popover.set_position(gtk::PositionType::Top);
        popover.set_has_arrow(true);

        // todo: investigate setting this to false breaks motion controller, but setting true
        // causes popup to be hidden twice. see search_bar connect_has_focus_notify spam in console.
        popover.set_autohide(true);
        
        let popover_connect_show = popover.clone();
        let obj_connect_show = obj.clone();
        let volume_icon_connect_show = volume_icon.clone();

        let volume_scale = gtk::Scale::with_range(
            gtk::Orientation::Horizontal, 
            0.0, 100.0, 10.0);
        let volume_scale_connect_show = volume_scale.clone();
        let popover_box = gtk::Box::new(gtk::Orientation::Vertical, 0);

        let application_window_connect_show = application_window.clone();
        popover.connect_show(move |_: &gtk::Popover| {
            volume_mixer::get_audio_registry();
            // must set this to have pointer events fire
            application_window_connect_show.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::OnDemand);
            popover_connect_show.queue_resize();
            let bounds = volume_icon_connect_show.compute_bounds(&application_window_connect_show).expect(
                "could not compute bounds of volume popover");
            popover_connect_show.set_offset(0, -bounds.y() as i32);
            obj_connect_show.add_css_class("focused-topbar-button");
            volume_scale_connect_show.grab_focus();
        });

        let application_window_connect_hide = application_window.clone();
        let focus_on_hide = focus_on_hide.clone();
        popover.connect_hide(move |_: &gtk::Popover| {
            println!("popover.connect_hide()");
            application_window_connect_hide.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive);
            focus_on_hide.grab_focus();
        });
  
        use gtk::prelude::ObjectExt;

        let obj_connect_close = obj.clone();
        popover.add_css_class("volume-popover");
        popover.connect_closed(move |_: &gtk::Popover | {
            obj_connect_close.remove_css_class("focused-topbar-button");
        });

        
        popover_box.add_css_class("popover-box");
        popover_box.append(&volume_scale);

        attach_popover_motion_controller(
            &popover.clone(), application_window.clone(), volume_icon, obj.clone());

        popover.set_child(Some(&popover_box));

        popover.set_can_target(true);
        popover.set_parent(&obj);    
        *volume_control.popover.borrow_mut() = Some(popover) ;
        obj
    }

    pub fn get(&self) -> &inner::VolumeControl {
        inner::VolumeControl::from_obj(self)
    }
}

fn attach_popover_motion_controller(
        popover: &gtk::Popover,
        application_window: gtk::ApplicationWindow, 
        volume_icon: gtk::Image,
        volume_control_obj: VolumeControl) {

    let popover_motion = popover.clone();
    let obj_motion = volume_control_obj.clone();
    let ecm = gtk::EventControllerMotion::builder()
        .propagation_phase(gtk::PropagationPhase::Capture)
        .build();

    ecm.connect_motion(move  |ecm: &gtk::EventControllerMotion, x, y| {
        let volume_control = &inner::VolumeControl::from_obj(&obj_motion);
        let volume_icon_bounds = volume_icon.compute_bounds(&application_window)
            .expect("failed to compute volume icon bounds");
        let popover_bounds = popover_motion.compute_bounds(&application_window)
            .expect("failed to compute volume icon bounds");
        if popover_bounds.width() == 0.0 || popover_bounds.height() == 0.0 {
            return;
        }
        let max_y = -popover_bounds.y() + volume_icon_bounds.y() + volume_icon_bounds.height() * 1.7;
        if y < 0.0 || x < 0.0 || x > popover_bounds.width().into() || y > max_y.into() {
            application_window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive);
            popover_motion.hide();
        }
    });
    popover.add_controller(ecm);
}