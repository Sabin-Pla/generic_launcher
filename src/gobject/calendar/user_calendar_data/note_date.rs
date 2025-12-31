#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct NoteDate {
	pub year: u32,
	pub month: u32,
	pub day: u32
}

impl From<(u32, u32, u32)> for NoteDate {
    fn from(date: (u32, u32, u32)) -> Self {
        Self {
            year: date.0,
            month: date.1, 
            day: date.2
        }
    }
}


impl From<&str> for NoteDate {
    fn from(date: &str) -> Self {
        let parts: Vec<&str> = date.split("-").collect();
        assert!(parts.len() == 3, "NoteDate given invalid string: {date}");
        let parts: Vec<u32> = parts.into_iter().map(|p| p.parse::<u32>().unwrap()).collect();
        Self {
            year: parts[0],
            month: parts[1], 
            day: parts[2]
        }
    }
}

impl std::fmt::Display for NoteDate  {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

impl Ord for NoteDate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.year
            .cmp(&other.year)
            .then(self.month.cmp(&other.month))
            .then(self.day.cmp(&other.day))
    }
}

impl PartialOrd for NoteDate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(Ord::cmp(self, other))
    }
}