
## Generic Launcher

A (currently not finished/barely started) implementation of a generic applications launcher in Rust.
It uses the GTK layer shell protocol.

Currently is just a window that will hide and unhide when the program is called.

The idea is to bind it to the super key and use it as an applications launcher/control-center-panel-type thing. 

## Ordered todo:
- Workspace Indicators
- Run applications (.Desktop)
	-	Launcher entries have copy-able File Path (does not trigger quit accelerator)
	-	Right click context menu
		-	Has 3 options: Run, Open xdg desktop cofig file, Permit fullscreen
			- Permit fullscreen: Make program fullscreen-able in Hyprland (may need plugin dep)
				- add icon in entry node to show program preference
				- add conf file to read in preference
				- support general styling in conf file
- File querying
