use crate::OsStr;

mod xdg_desktop_entry;
pub use xdg_desktop_entry::XdgDesktopEntry;

pub fn get_xdg_desktop_entries() -> (Vec<XdgDesktopEntry>, Vec<XdgDesktopEntry>) {
    let home = std::env::var("HOME").unwrap_or("~".to_string());
    let dirs_entries =
        std::env::var("XDG_DATA_DIRS").unwrap_or("/usr/local/share:/usr/share".to_string());
    let data_home = std::env::var("XDG_DATA_HOME").unwrap_or(home + "/.local/share");
    let applications_folders = [data_home.split(':'), dirs_entries.split(':')]
        .into_iter()
        .flatten()
        .map(|mut d| {
            let mut d2 = d.chars();
            if let Some('/') = d2.next_back() {
                d = d2.as_str(); // remove ending '/' if present
            }
            d.to_owned() + "/applications"
        });

    let mut added = vec![];
    let mut launcher_files: Vec<_> = applications_folders
        .filter_map(|folder| {
            println!("{:?}", folder);
            if !added.contains(&folder) {
                // filter duplicates (if folder is in both env vars)
                added.push(folder.clone());
                return Some(std::path::Path::new(&folder).read_dir());
            }
            None
        })
        .collect();

    let mut custom_launcher = std::env::current_dir().expect("Error accessing CWD");
    custom_launcher.push("misc");
    launcher_files.push(custom_launcher.read_dir());

    let desktop_extension = Some(OsStr::new("desktop"));
    let mut entries: Vec<XdgDesktopEntry> = vec![];
    let mut custom_entries: Vec<XdgDesktopEntry> = vec![];

    for path in launcher_files {
        let path: std::fs::ReadDir = match path {
            Err(..) => continue,
            Ok(p) => p,
        };
        let contents = path.map(|p| p.unwrap().path());
        for path in contents {
            if path.extension() == desktop_extension {
                let entry = XdgDesktopEntry::try_from(&path);
                if let Some(entry) = entry {
                    if let Some(_) = entry.app_info.locale_string("GenericLauncherCustom") {
                        custom_entries.push(entry);
                        continue;
                    }
                    entries.push(entry);
                    continue;
                }
                println!("could not create launcher for path {:?}", path);
            }
        }
    }
    println!("{:#?}", entries);
    (entries, custom_entries)
}
