#[derive(Debug)]
pub enum KeyboardMode {
	Exclusive,
	OnDemand
}

impl Default for KeyboardMode {
	fn default() -> Self {
		Self::Exclusive
	}
}

impl From<&str> for KeyboardMode {
	fn from(s: &str) -> Self {
		match s.to_lowercase().as_str() {
			"exclusive" => Self::Exclusive,
			"on_demand" => Self::OnDemand,
			_ => panic!("bad keyboard mode")
		}
	}
}