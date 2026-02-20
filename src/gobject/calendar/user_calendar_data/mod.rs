mod note_data;
mod note_date;

pub use note_data::NoteData;
pub use note_date::NoteDate;

pub use super::{MarkingType};
use std::collections::BTreeMap;

pub struct UserCalendarData {
	notes: BTreeMap<NoteDate, NoteData>,
    calendar_data_file: (std::path::PathBuf, std::fs::File)
}

impl UserCalendarData {
	pub fn load() -> Self {
		let mut calendar_data_file = get_user_calendar_data_file().expect("error reading calendar data file");
		Self {
			notes: parse_user_file(&mut calendar_data_file.1),
            calendar_data_file
		}
	}

	pub fn get_date_entry(&mut self, note_date: NoteDate) -> Option<&NoteData> {
		self.notes.get(&note_date)
	}

    pub fn get_or_insert_date_entry(&mut self, note_date: NoteDate) -> &mut NoteData {
        self.notes.entry(note_date).or_insert(NoteData::default())
    }

    pub fn get_date_notes(&self, note_date: NoteDate) -> Option<&str> {
        match self.notes.get(&note_date) {
            Some(note_date) => Some(note_date.note.as_str()),
            None => None
        }
    }

    pub fn resync_note_changes(&mut self, note_date: NoteDate, note: String) {
        let entry =  self.get_or_insert_date_entry(note_date);
        if entry.note != note.as_str() {
            entry.note = note;
            println!("resync_note_changes {note_date}");
            self.write_contents();
        }
    }

    pub fn write_contents(&self) {
        let toml = toml::Table::from_iter(
            self.notes.iter().map(|(note_date, note_data)| {
                let mut table = toml::Table::new();
                table.insert("marking".to_string(), toml::Value::String(note_data.marking.to_string()));
                table.insert("note".to_string(), toml::Value::String(note_data.note.clone()));
                (note_date.to_string(), toml::Value::Table(table))
        }));
        use std::io::Write;
        let file_path = self.calendar_data_file.0.clone();
        std::thread::spawn(move || {
            let tmp_path = file_path.with_extension("tmp");
            let mut tmp_file = std::fs::File::create(&tmp_path)
                .expect("could not create temp file for writing calendar user data");
            tmp_file.write_all(toml.to_string().as_bytes())
                .expect("could not write to calendar user data temp file");
            tmp_file.sync_all().expect("failed to flush temp calendar user data swap file to disk");
            std::fs::rename(tmp_path, file_path)
                .expect("failed to overwrite calendar user data file"); 
        });
    }
}

fn get_user_calendar_data_file() -> Result<(std::path::PathBuf, std::fs::File), Box<dyn std::error::Error>>  {
    let is_writable = |path: &str| {
        std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
    };

    let calendar_data_file_path = match std::env::var_os("XDG_DATA_HOME") {
        Some(data_dir) => {
            let mut path = std::path::PathBuf::from(&data_dir.into_string().unwrap());
            path.push("generic_launcher");
            let _ = std::fs::create_dir_all(&path);
            path.push("calendar_data.toml");
            path
        },
        None => {
            let mut path = std::env::home_dir()
                .expect("Could not get home directory fallback for unprovided XDG_DATA_HOME");
            path.push(".local");
            path.push("share");
            path.push("generic_launcher");
            std::fs::create_dir_all(&path)?;
            path.push("calendar_data.toml");
            path
        }
    };

    let path_str = calendar_data_file_path.to_str().unwrap();
    if let Ok(calendar_data_file) = is_writable(path_str) {
        Ok((calendar_data_file_path, calendar_data_file))
    } else {
        panic!("Could not write to fallback datapath $HOME/.local/share/generic_launcher")
    }
}

fn parse_user_file(mut file: &std::fs::File) -> BTreeMap<NoteDate, NoteData> {
    let mut notes_map = BTreeMap::new();
    let mut contents = vec!();
    use std::io::Read;
    file.read_to_end(&mut contents).expect("failure reading calendar user data file");
    let contents = std::str::from_utf8(&contents).expect("failure parsing calendar user data as utf8 string");
    let contents = contents.parse::<toml::Table>().expect(
        &format!("failed to parse calendar data file as toml ({:?})", file));
    for (date, notes) in contents.iter() {
        let note_data = match notes {
             toml::Value::Table(notes) => {
                let marking = if let Some(marking) = notes.get("marking") {
                    MarkingType::from(marking.as_str().expect(
                        &format!("note marking on date {date} is not a string")))
                } else {
                    MarkingType::None
                };
                let note = if let Some(note) = notes.get("note") {
                    note.as_str().expect(
                        &format!("notes on date {date} is not a string"))
                } else {
                    ""
                };
                NoteData { marking, note: note.to_string() }
            }
            _ => panic!("malformed toml entry for calendar data at date {date}")
        };
        notes_map.insert(NoteDate::from(date.as_str()), note_data);
    }
    notes_map
}