use std::convert::AsRef;
use std::env;
use crate::{Path, PathBuf};

use super::UserConfig;

#[derive(Debug)]
pub struct ApplicationSettings {
	pub css_file: gio::File,
	pub icons_file: gio::File,
    pub user_config: UserConfig
}

impl ApplicationSettings {
	pub fn load() -> Self {
		Self {
			css_file: css_file(),
			icons_file: icons_file(),
			user_config: UserConfig::load()
		}
	}
}

fn css_file() -> gio::File {
	let css_subpath: PathBuf = "launcher.css".into();
    let mut css_path = glib::user_config_dir();
    css_path.push("generic_launcher");
    css_path.push(&css_subpath);
    get_or_symlink(css_path, css_subpath)
}

fn icons_file() -> gio::File {
	let icon_theme_subpath: PathBuf  = ["generic_launcher", "assets"].iter().collect();
	let installed_icon_path = glib::user_data_dir().into_iter()
		.chain(&icon_theme_subpath)
		.collect();
	get_or_symlink(installed_icon_path, PathBuf::from("assets"))
}

fn get_or_symlink<P: AsRef<Path>>(path: P, fallback: P) -> gio::File {
	match path.as_ref().exists() {
        true => gio::File::for_path(path),
        false => {
        	let install_dir: PathBuf  = env::var("GENERIC_LAUNCHER_INSTALL_DIR")
        		.expect("GENERIC_LAUNCHER_INSTALL_DIR variable must be provided on initialization")
        		.into();
            let fallback: PathBuf = install_dir.into_iter()
            	.chain(fallback.as_ref().iter())
            	.collect();

			match std::fs::symlink_metadata(path.as_ref()) {
				Ok(metadata) => {
					if metadata.is_symlink() {
						std::fs::remove_file(path.as_ref());
					}
				}
				Err(..) => (),
			}
			let fallback = fallback.canonicalize().expect(
				&format!("failed to cononicalize path {:?}", fallback));
            let _ = std::fs::create_dir_all(path.as_ref().parent().expect(
            	"Failed to get parent dir of path"));
            let _ = std::os::unix::fs::symlink(&fallback, &path).expect(
            	"failed to make symlink");
            println!("symlinked path {:?} {:?}", path.as_ref(), fallback.into_os_string());
          	gio::File::for_path(path)
        }
    }
}