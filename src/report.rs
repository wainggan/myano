
pub struct Report {
	fault: bool,
	errors: Vec<String>,
	warnings: Vec<String>,
}
impl Report {
	pub fn new() -> Self {
		Self {
			fault: false,
			errors: Vec::new(),
			warnings: Vec::new(),
		}
	}

	pub fn error(&mut self, msg: String) {
		self.errors.push(msg);
		self.fault = true;
	}

	pub fn warn(&mut self, msg: String) {
		self.warnings.push(msg);
	}

	pub fn ok(&self) -> bool {
		!self.fault
	}
}
impl std::fmt::Display for Report {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "report! {{ errors: {:?}, warnings: {:?} }}", self.errors, self.warnings)
	}
}
impl std::fmt::Debug for Report {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		std::fmt::Display::fmt(&self, f)
	}
}
impl std::error::Error for Report {}

