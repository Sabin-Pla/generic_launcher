use std::cell::RefCell;
use std::rc::Rc;

use gtk::glib::{Object};
use gtk::prelude::{Cast, LayoutManagerExt, WidgetExt, BoxExt, PopoverExt};
use gtk::subclass::prelude::*;
use gdk::Rectangle;

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
    pub fn new(icon_theme: &gtk::IconTheme, application_window: &gtk::ApplicationWindow, attachment: &gtk::Box) -> Self {
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
        popover.set_has_arrow(false);
        popover.set_autohide(false);

        use gtk::prelude::GestureDragExt;
        let drag = gtk::GestureDrag::builder()
            .propagation_phase(gtk::PropagationPhase::Capture)
            .build();
        drag.connect_drag_begin(|g: &gtk::GestureDrag, _, _| {
            println!("ooo");
        });

        let click = gtk::GestureClick::builder()
            .propagation_phase(gtk::PropagationPhase::Target)
            .build();

        use gtk::prelude::GestureExt;
        click.connect_pressed(|g: &gtk::GestureClick, _, _, _| {
            g.set_state(gtk::EventSequenceState::Claimed);
            println!("oocco");
        });
        

        let popover_connect_show = popover.clone();
        let obj_connect_show = obj.clone();
        let volume_icon_connect_show = volume_icon.clone();

        let volume_scale = gtk::Scale::with_range(
            gtk::Orientation::Horizontal, 
            0.0, 100.0, 10.0);
        let s = volume_scale.clone();
        let popover_box = gtk::Box::new(gtk::Orientation::Vertical, 0);

        let pb = popover_box.clone();
        popover.connect_show(move |_: &gtk::Popover| {
           // popover_connect_show.set_parent(&application_window);
            popover_connect_show.queue_resize();
          //  let bounds = volume_icon_connect_show.compute_bounds(application_window).expect(
            //    "could not compute bounds of volume popover");
           // println!("{:?} {:?}", bounds, popover_connect_show.measure());
            //popover_connect_show.set_offset(0, (bounds.y() / 1.5) as i32);
             popover_connect_show.set_offset(0, 20);
            popover_connect_show.set_pointing_to(Some(&gdk::Rectangle::new(0, 0, 50, 50)));
            obj_connect_show.add_css_class("focused-topbar-button");
            s.grab_focus();
            println!("popover allocation: {:?}", popover_connect_show.allocation());
            println!("popover_box allocation: {:?}", pb.allocation());
        });
        popover.connect_map(|p| {
            println!("popover mapped, allocation: {:?}", p.allocation());
        });
        use gtk::prelude::ObjectExt;
        let po = popover.clone();
        popover.connect_notify_local(Some("allocation"), move |_, _| {
            println!("popover allocation changed: {:?}", po.allocation());
        });
        let obj_connect_close = obj.clone();
        popover.add_css_class("volume-popover");
        popover.connect_closed(move |_: &gtk::Popover | {
            obj_connect_close.remove_css_class("focused-topbar-button");
        });

        
        //popover.set_layout_manager(Some(gtk::BinLayout::new()));
        // volume_scale.set_focus_on_click(true);
        //popover_box.append(&volume_scale);
        volume_scale.set_focusable(true);
        popover.set_focusable(true);
        popover_box.set_focusable(true);
        volume_scale.set_can_focus(true);
        popover.set_can_focus(true);
        popover_box.set_can_focus(true);
        popover_box.set_hexpand(true);
        popover_box.set_vexpand(true);
        popover_box.add_css_class("popover-box");
        popover_box.set_size_request(100, 40);
        popover.set_size_request(100, 40);
        // popover.set_cascade_popdown(true);
        // popover.set_child(Some(&volume_scale));

        popover.set_hexpand(true);
        popover.set_vexpand(true);
        popover.set_default_widget(Some(&popover_box));

        let ecm = gtk::EventControllerMotion::builder()
            .propagation_phase(gtk::PropagationPhase::Capture)
            .build();

        ecm.connect_enter(|_, _, _| {
            println!("MOTION");
        });

        use gtk::prelude::ButtonExt;
        //popover_box.add_controller(click);
        popover.add_controller(click);
        popover.set_size_request(200, 40);
        let button = gtk::Button::with_label("Click me");
        button.connect_clicked(|_| println!("clicked!"));
        //popover_box.append(&button);
        popover.set_child(Some(&button));
        use gtk::prelude::WidgetExt;
        popover.set_can_target(true);
        obj.set_can_target(true);
        popover_box.set_can_target(true);
        let po = popover.clone();
        *volume_control.popover.borrow_mut() = Some(popover) ;
        po.set_parent(&obj);    
        obj
    }

    pub fn get(&self) -> &inner::VolumeControl {
        inner::VolumeControl::from_obj(self)
    }
}
