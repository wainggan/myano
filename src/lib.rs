
pub mod report;
pub mod token;
pub mod parse;
pub mod bind;


pub use token::tokenize;
pub use parse::parse;


fn main() {
	let src = "1 + 1";
	let tokens = tokenize(src).unwrap();
	let ast = parse(src, &tokens).unwrap();
}

