
## Generic Launcher

A generic applications launcher. 
Supports css styling through css file created in `$XDG_CONFIG_HOME/generic_launcher/launcher.css`

Depends on `gtk4-layer-shell` (https://github.com/wmww/gtk4-layer-shell) and `inotifytools` (todo: make this dependency optional for hotswapping css)

Screenshot button depends on `grim`, `wl-clipboard`, and `slurp` 

## Installation

Install the dependencies, listed above. Then, change directory into repo root and run
`./install.sh <install destination directory>` 

Set up shortcuts for your compositor. For Hyprland I use: 

`$launcher = (cd <repo folder> && <repo folder>/target/debug/generic_launcher)`
`bind = SUPER, SUPER_L, exec, $launcher`

#### Todo list:
- Scroll bar on search results
- Settings toml file logic is partially implemented but unused and untested
- Set window size properly for HiDPI displays
- Launcher options on dropdown/rightclick, as some .desktop files have multiple launch options
  - Also add "Open .desktop file location" option there
- Allow selecting from a set of pre-defined themes. 
  - Not sure how application will look on a variety of system themes as I only ever use Andromeda for the app. needs more testing.
- Install procedure is a bit janky
  - Requires application be launched once on install
  - Symlinks from user's `$XDG_CONFIG_HOME` to install directory launcher.css, assets file are created, copy the file over for release builds  

#### Other features that may be added in the future:
- Dictionary lookup feature
  - Clean up IMController placeholder class after dictionary lookup feature  
- Icons? (not sure how it will look until it's implemented, maybe don't need this.)
- Settings page 
  - Creating/integrating a seperate repo that can create a gtk window from an annotated toml file to display application settings sounds like an epic idea 
- System controls (power button, logout)
- Volume mixer and audio input/output device selector 
- System indicators for applications that unmap windows (may have to be restricted to hyprland or some specific set of compositors)

![Demo:](docs/demo_screenshot.jpg)

The default CSS used in the screenshot uses the default system theme, in this case [Andromeda](https://www.gnome-look.org/p/2039961)

## Attributions

Thank you to the gnome project http://www.gnome.org for icons (Adwaita)
These are packaged to avoid dependencies.


