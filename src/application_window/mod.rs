mod initialize_widgets;
mod event_handler;

use crate::Path;

use gtk::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use gtk::PropagationPhase;

use crate::ApplicationSettings;
use crate::launcher::Launcher;

pub fn initialize(application: &gtk::Application) -> gtk::ApplicationWindow {
    let application_window = gtk::ApplicationWindow::new(application);
    let action_close = gio::ActionEntry::builder("close")
    	.activate(|w: & gtk::ApplicationWindow, _, _| { w.close(); })
    	.build();

    application_window.add_action_entries([action_close]);
    application_window.init_layer_shell();

    // todo!("make these pixel values proportional");
    application_window.set_layer(Layer::Overlay);
    application_window.set_margin(Edge::Left, 800);
    application_window.set_margin(Edge::Right, 800);
    application_window.set_margin(Edge::Top, 400);

    let anchors = [
        (Edge::Left, true),
        (Edge::Right, true),
        (Edge::Top, false),
        (Edge::Bottom, false),
    ];

    for (anchor, state) in anchors {
        application_window.set_anchor(anchor, state);
    }
    application_window
}   

pub fn populate(
        application_window: &mut gtk::ApplicationWindow, 
        application_settings: &ApplicationSettings,
        launcher: &mut Launcher) {

    let icon_theme = get_icon_theme(&application_settings);
    setup_event_controllers(application_window);
	initialize_widgets::root(application_window, launcher, &icon_theme);
}

pub fn setup_event_controllers(application_window: &mut gtk::ApplicationWindow) {
    let ec = gtk::EventControllerKey::builder()
        .propagation_phase(PropagationPhase::Capture).build();
    ec.connect_key_pressed(event_handler::key_handler);
    application_window.add_controller(ec);
}


pub fn get_icon_theme(application_settings: &ApplicationSettings) -> gtk::IconTheme {
    let icon_theme = gtk::IconTheme::builder()
        .theme_name("Adwaita")
        .build();
    let resource_path = application_settings.icons_file
        .path()
        .expect("Failed to parse path for icon file");
    let resource_path = resource_path.to_str().expect("Error converting icon theme resource path to string");
    println!("icon theme resource path: {:?}", resource_path);
    icon_theme.set_resource_path(&[&resource_path]);
    icon_theme.set_search_path(&[Path::new(&resource_path)]);
    icon_theme
}