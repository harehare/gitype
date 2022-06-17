#[derive(Clone, Debug)]
pub struct Line {
	line_no: usize,
	head_space: Option<String>,
	entered: Option<String>,
	current: Option<char>,
	rest: Option<String>,
}

impl Line {
	fn start_index(line: &str) -> usize {
		line.len() - line.trim_start().len()
	}

	pub fn new(line_no: usize, line: &str) -> Self {
		let text = &line[Line::start_index(line)..];
		let head_space = &line[0..Line::start_index(line)];
		let head_space = if head_space.is_empty() {
			None
		} else {
			Some(head_space.to_string())
		};

		match text.chars().collect::<Vec<char>>().as_slice() {
			[h] => Line {
				line_no: line_no,
				head_space: head_space,
				entered: None,
				current: Some(h.clone()),
				rest: None,
			},
			[h, rest @ ..] => Line {
				line_no: line_no,
				head_space: head_space,
				entered: None,
				current: Some(h.clone()),
				rest: Some(String::from_iter(rest)),
			},
			_ => Line {
				line_no: line_no,
				head_space: None,
				entered: None,
				current: None,
				rest: None,
			},
		}
	}

	pub fn current_text(&self) -> Option<char> {
		self.current
	}

	pub fn entered_text(&self) -> Option<String> {
		match (self.head_space.clone(), self.entered.clone()) {
			(Some(h), Some(entered)) => Some(h + &entered),
			(Some(h), None) => Some(h),
			(None, Some(entered)) => Some(entered),
			_ => None,
		}
	}

	pub fn rest_text(&self) -> Option<String> {
		self.rest.clone()
	}

	pub fn input(&self, c: char) -> bool {
		match self.current {
			Some(i) => i == c,
			None => true,
		}
	}

	pub fn line_no(&self) -> usize {
		self.line_no
	}

	pub fn is_entered(&self) -> bool {
		self.rest.is_none()
	}

	pub fn next(&self) -> Self {
		if let Some(rest) = self.rest.clone() {
			match rest.chars().collect::<Vec<char>>().as_slice() {
				[h, rest @ ..] => Line {
					line_no: self.line_no,
					head_space: self.head_space.clone(),
					entered: match self.entered.clone() {
						Some(e) => self.current.map(|c| e + String::from(c).as_str()),
						None => self.current.map(String::from),
					},
					current: Some(h.clone()),
					rest: Some(String::from_iter(rest)),
				},
				_ => Line {
					line_no: self.line_no,
					head_space: self.head_space.clone(),
					entered: match self.entered.clone() {
						Some(e) => self.current.map(|c| e + String::from(c).as_str()),
						None => self.current.map(String::from),
					},
					current: None,
					rest: None,
				},
			}
		} else {
			self.clone()
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn new() {
		let line = Line::new(1, "    {");
		assert_eq!(line.head_space.unwrap(), "    ");
	}

	#[test]
	fn has_next() {
		let input = Line::new(1, "      input test");
		assert_eq!(
			input
				.clone()
				.current_text()
				.map(String::from)
				.unwrap_or("".to_owned()),
			"i"
		);

		let next_input = input.next();

		assert_eq!(
			&next_input.head_space.clone().unwrap(),
			&"      ".to_string()
		);
		assert_eq!(
			&next_input
				.current_text()
				.map(String::from)
				.unwrap_or("".to_owned()),
			&"n"
		);
		assert_eq!(&next_input.entered_text().unwrap(), &"      i".to_string());
		assert_eq!(&next_input.rest_text().unwrap(), &"put test".to_string());
	}

	#[test]
	fn no_next() {
		let input = Line::new(0, "i");
		assert_eq!(
			input
				.clone()
				.current_text()
				.map(String::from)
				.unwrap_or("".to_owned()),
			"i"
		);
		let next_input = input.next();
		assert!(next_input.is_entered());
	}

	#[test]
	fn new_line_only() {
		let input = Line::new(0, "\n");
		assert_eq!(
			input
				.clone()
				.current_text()
				.map(String::from)
				.unwrap_or("".to_owned()),
			""
		);
		let next_input = input.next();
		assert!(next_input.is_entered());
	}
}
