use hyprland::data::{Client, Clients, Monitors, Workspace};
use hyprland::dispatch::*;
use hyprland::event_listener::EventListener;
use hyprland::keyword::*;
use hyprland::prelude::*;
use hyprland::shared::WorkspaceType;
use hyprland::shared::HyprError;
use hyprland::event_listener::AsyncEventListener;

struct CompositorInfo {
	activeWindow: String,
}

pub fn draw_titlebars() -> hyprland::Result<()>  {
    tokio::runtime::Builder::new_multi_thread()
    	.enable_io()
    	.enable_time()
    	.worker_threads(1)
    	.build()?.block_on(async move {
    		let mut event_listener = AsyncEventListener::new();
        	// hyprland::dispatch!(async; Exec, "kitty").await?;
        	event_listener.add_workspace_change_handler(
	            async_closure! { move |id| println!("workspace changed to {id:#?}")},
	        );
	        event_listener.add_active_window_change_handler(
	            async_closure! { move |data| println!("window changed to {data:#?}")},
			);
			event_listener.start_listener_async().await
    })
}