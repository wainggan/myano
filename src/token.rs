
use crate::report::Report;

use std::marker::PhantomData;


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TT {
	Eof,

	True,
	False,
	Identifier,
	Integer,
	Float,
	
	Plus, // +
	Minus, // -
	Star, // *
	Slash, // /

	Bang, // !

	Equal, // =
	EqualEqual, // ==
	BangEqual, // !=
	Lesser, // <
	Greater, // >
	LesserEqual, // <=
	GreaterEqual, // >=
	
	LParen, // (
	RParen, // )
	LBracket, // [
	RBracket, // ]
	LBrace, // {
	RBrace, // }

	Dot, // .
	Comma, // ,

	Colon, // :
	SemiColon, // ;

	Let, // let
	Mut, // mut

	If, // if
	Else, // else
	For, // for
	While, // while
	Loop, // loop

	Export, // export

	Struct, // struct
	Module, // module
	Fn, // fn
	EqualGreater, // =>
}

#[derive(Clone)]
pub struct Token {
	pub kind: TT,
	src: (u32, u32),
}
impl Token {
	pub fn new(kind: TT, src: (u32, u32)) -> Self {
		Self {
			kind,
			src,
		}
	}
}
impl std::fmt::Display for Token {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "({:?} : {}..{})", self.kind, self.src.0, self.src.1)
	}
}
impl std::fmt::Debug for Token {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		std::fmt::Display::fmt(&self, f)
	}
}

#[derive(Debug)]
pub struct TokenStream<'a> {
	pub src: &'a str,
	pub tokens: Vec<Token>,
}
impl<'a> TokenStream<'a> {
	pub fn new(src: &'a str, tokens: Vec<Token>) -> Self {
		Self {
			src,
			tokens,
		}
	}

	pub fn get(&self, index: usize) -> &Token {
		&self.tokens[index]
	}

	pub fn str_from(&self, token: &Token) -> &str {
		&self.src[token.src.0 as usize..token.src.1 as usize]
	}
}


struct Tokenize<'a> {
	src: &'a str,
	tokens: Vec<Token>,
	report: Report,
	start: usize,
	current: usize,
}
impl<'a> Tokenize<'a> {
	fn new(src: &'a str) -> Self {
		Self {
			src,
			tokens: Vec::new(),
			report: Report::new(),
			start: 0,
			current: 0,
		}
	}

	fn advance(&mut self, iter: &mut impl Iterator<Item = char>) {
		iter.next();
		self.current += 1;
	}

	fn build(mut self) -> Result<TokenStream<'a>, Report> {
		let mut iter = self.src.chars().peekable();

		while let Some(c) = iter.next() {
			self.start = self.current;
			self.current += 1;
			
			match c {
				'+' => self.add(TT::Plus),
				'-' => self.add(TT::Minus),
				'*' => self.add(TT::Star),
				'/' => self.add(TT::Slash),

				'=' => match iter.peek() {
					Some('=') => {
						self.advance(&mut iter);
						self.add(TT::EqualEqual);
					},
					Some('>') => {
						self.advance(&mut iter);
						self.add(TT::EqualGreater);
					},
					_ => self.add(TT::Equal),
				},

				'(' => self.add(TT::LParen),
				')' => self.add(TT::RParen),
				'[' => self.add(TT::LBracket),
				']' => self.add(TT::RBracket),
				'{' => self.add(TT::LBrace),
				'}' => self.add(TT::RBrace),

				'.' => self.add(TT::Dot),
				',' => self.add(TT::Comma),

				':' => self.add(TT::Colon),
				';' => self.add(TT::SemiColon),
				
				_ => {
					if c.is_whitespace() {
						// ignore
					} else if c.is_numeric() {

						while let Some(c) = iter.peek() {
							if !c.is_numeric() {
								break;
							}
							self.advance(&mut iter);
						}
						if iter.peek() == Some(&'.') {
							self.advance(&mut iter);
							while let Some(c) = iter.peek() {
								if !c.is_numeric() {
									break;
								}
								self.advance(&mut iter);
							}
							self.add(TT::Float);
						} else {
							self.add(TT::Integer);
						}

					} else if c.is_alphabetic() {
						while let Some(c) = iter.peek() {
							if !c.is_alphanumeric() {
								break;
							}
							self.advance(&mut iter);
						}
						let check = &self.src[self.start..self.current];
						match check {
							"true" => self.add(TT::True),
							"false" => self.add(TT::False),
							"if" => self.add(TT::If),
							"else" => self.add(TT::Else),
							"let" => self.add(TT::Let),
							"mut" => self.add(TT::Mut),
							"fn" => self.add(TT::Fn),
							_ => self.add(TT::Identifier),
						}
					} else {
						self.report.error(format!("unknown character '{}' at {}", c, self.start));
					}
				},
			}
		}

		self.eof();

		if self.report.ok() {
			Ok(TokenStream {
				src: self.src,
				tokens: self.tokens,
			})
		} else {
			Err(self.report)
		}
	}

	fn add(&mut self, kind: TT) {
		self.tokens.push(Token::new(kind, (self.start as u32, self.current as u32)));
	}

	fn eof(&mut self) {
		self.tokens.push(Token::new(TT::Eof, (0, 0)));
	}

}


pub fn tokenize<'a>(src: &'a str) -> Result<TokenStream<'a>, Report> {
	Tokenize::new(src).build()
}


#[cfg(test)]
mod test {
    use crate::token::*;

	#[test]
	fn symbols() {
		let src = "+ - * / = ==";
		let tokens = tokenize(src).unwrap();
		assert_eq!(
			tokens.iter().map(|v| v.kind).collect::<Vec<_>>(),
			vec![
				TT::Plus, TT::Minus,
				TT::Star, TT::Slash,
				TT::Equal, TT::EqualEqual,
				TT::Eof,
			],
		);
	}

	#[test]
	fn numbers() {
		let src = "1 10 100 1. 1.0 100.000";
		let tokens = tokenize(src).unwrap();
		assert_eq!(
			tokens.iter().map(|v| v.kind).collect::<Vec<_>>(),
			vec![
				TT::Integer, TT::Integer, TT::Integer,
				TT::Float, TT::Float, TT::Float,
				TT::Eof
			],
		);
	}

	#[test]
	fn get() {
		let src = "+ - * / 100 1 1.0 1. 10.00";
		let tokens = tokenize(src).unwrap();
		assert_eq!(
			tokens.iter().map(|v| v.get(src)).collect::<Vec<_>>(),
			vec!["+", "-", "*", "/", "100", "1", "1.0", "1.", "10.00", ""],
		);
	}
}

