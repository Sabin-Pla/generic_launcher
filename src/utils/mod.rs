pub mod hot_reload;
use std::cell::RefMut;

pub fn char_position(string: &'_ str, n: usize) -> usize {
	let mut counter = (0, 0);
	// gets the byte position of the nth character in a str
	for c in string.chars() {
		counter.1 += 1;
		counter.0 += c.len_utf8();
		if counter.1 == n {
			return counter.0;
		}
	}
	0
}