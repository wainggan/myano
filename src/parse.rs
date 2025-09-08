
use crate::{report::Report, token::{Token, TokenStream, TT}};

use std::{iter::Peekable, slice::Iter};


#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeIndex(pub u32);

#[derive(Debug)]
pub enum Node<'a> {
	Error,
	Module {
		root: NodeIndex,
	},
	Block {
		expr: Vec<NodeIndex>,
	},
	Identifier {
		name: &'a Token,
	},
	Bool {
		value: bool,
	},
	Integer {
		value: &'a Token,
	},
	Float {
		value: &'a Token,
	},
	Fn {
		args: Vec<(&'a Token, Option<NodeIndex>)>,
		ret: Option<NodeIndex>,
		expr: NodeIndex,
	},
	Group {
		expr: NodeIndex,
	},
	Binary {
		left: NodeIndex,
		op: &'a Token,
		right: NodeIndex,
	},
	Unary {
		op: &'a Token,
		right: NodeIndex,
	},
	Call {
		op: &'a Token,
		expr: NodeIndex,
		args: Vec<NodeIndex>,
	},
	If {
		op: &'a Token,
		condition: NodeIndex,
		then_branch: NodeIndex,
		else_branch: Option<NodeIndex>,
	},
	Let {
		mutable: bool,
		name: &'a Token,
		expr: NodeIndex,
		annotation: Option<NodeIndex>,
	},
}

#[derive(Debug)]
pub struct Ast<'a> {
	pub tokens: &'a TokenStream<'a>,
	pub nodes: Vec<Node<'a>>,
	pub root: NodeIndex,
}
impl<'a> Ast<'a> {
	pub fn get(&self, node: &NodeIndex) -> &Node {
		&self.nodes[node.0 as usize]
	}
}


struct Parser<'a> {
	src: &'a str,
	tokens: &'a TokenStream<'a>,
	nodes: Vec<Node<'a>>,
	iter: Peekable<Iter<'a, Token>>,
	report: Report,
}
impl<'a> Parser<'a> {
	fn new(src: &'a str, tokens: &'a TokenStream) -> Self {
		Self {
			src,
			tokens,
			nodes: Vec::new(),
			iter: tokens.tokens.iter().peekable(),
			report: Report::new(),
		}
	}

	fn add(&mut self, value: Node<'a>) -> NodeIndex {
		self.nodes.push(value);
		NodeIndex(self.nodes.len() as u32 - 1)
	}

	// since there is a TT::Eof at the end of the iterator, it's probably okay
	// unwrap, as nothing should ever end up consuming TT::Eof
	fn next(&mut self) -> &'a Token {
		self.iter.next().unwrap()
	}
	fn peek(&mut self) -> &'a Token {
		self.iter.peek().unwrap()
	}

	fn catch(&mut self, check: &[TT]) -> Option<&'a Token> {
		let kind = self.iter.peek()?.kind;
		if check.iter().find(|v| **v == kind).is_some() {
			self.iter.next()
		} else {
			None
		}
	}

	fn build(mut self) -> Result<Ast<'a>, Report> {
		let root = self.module();
		if self.report.ok() {
			Ok(Ast { tokens: self.tokens, nodes: self.nodes, root })
		} else {
			Err(self.report)
		}
	}

	fn module(&mut self) -> NodeIndex {
		let root = self.block(|_| false);
		self.add(Node::Module { root })
	}

	fn block(&mut self, end: impl Fn(TT) -> bool) -> NodeIndex {
		let mut expr = Vec::new();

		while let Some(c) = self.iter.peek() {
			if c.kind == TT::Eof {
				break;
			} else if end(c.kind) {
				self.iter.next();
				break;
			}
			expr.push(self.statement());
			self.catch(&[TT::SemiColon]);
		}

		self.add(Node::Block { expr })
	}

	fn statement(&mut self) -> NodeIndex {
		if let Some(op) = self.catch(&[TT::Let, TT::Mut]) {
			let name = self.catch(&[TT::Identifier]).unwrap();

			let annotation =
				if let Some(_) = self.catch(&[TT::Colon]) {
					Some(self.type_expression())
				} else {
					None
				};
			
			self.catch(&[TT::Equal]).unwrap();
			
			let expr = self.expression();
			
			self.add(Node::Let { mutable: op.kind == TT::Mut, name, annotation, expr })
		} else {
			self.expression()
		}
	}

	fn expression(&mut self) -> NodeIndex {
		self.function()
	}

	fn function(&mut self) -> NodeIndex {
		if let Some(_) = self.catch(&[TT::Fn]) {
			self.catch(&[TT::LParen]).unwrap();

			let mut args = Vec::new();
			loop {
				if let Some(_) = self.catch(&[TT::RParen]) {
					break;
				}
				
				let name = self.catch(&[TT::Identifier]).expect(&format!("found {}", self.peek()));

				let annotation;
				if let Some(_) = self.catch(&[TT::Colon]) {
					annotation = Some(self.type_expression());
				} else {
					annotation = None;
				}
				
				args.push((name, annotation));

				self.catch(&[TT::Comma]);
			}

			let ret =
				if let Some(_) = self.catch(&[TT::Colon]) {
					Some(self.type_expression())
				} else {
					None
				};

			if let None = self.catch(&[TT::EqualGreater]) {
				let tt = self.peek();
				self.report.error(format!("expected '=>', found {:?}", tt));
			}

			let expr = self.expression();

			self.add(Node::Fn { args, ret, expr })
		} else {
			self.jump()
		}
	}

	fn jump(&mut self) -> NodeIndex {
		if let Some(op) = self.catch(&[TT::If]) {
			let condition = self.equality();
			
			let then_branch = self.expression();

			let else_branch =
				if let Some(_) = self.catch(&[TT::Else]) {
					Some(self.expression())
				} else {
					None
				};
			
			self.add(Node::If { op, condition, then_branch, else_branch })
		} else {
			self.equality()
		}
	}

	fn equality(&mut self) -> NodeIndex {
		let mut left = self.term();
		while let Some(op) = self.catch(&[
			TT::EqualEqual, TT::BangEqual,
			TT::Lesser, TT::LesserEqual,
			TT::Greater, TT::GreaterEqual,
		]) {
			let right = self.term();
			left = self.add(Node::Binary { left, op, right });
		}
		left
	}

	fn term(&mut self) -> NodeIndex {
		let mut left = self.factor();
		while let Some(op) = self.catch(&[TT::Plus, TT::Minus]) {
			let right = self.factor();
			left = self.add(Node::Binary { left, op, right });
		}
		left
	}

	fn factor(&mut self) -> NodeIndex {
		let mut left = self.unary();
		while let Some(op) = self.catch(&[TT::Star, TT::Slash]) {
			let right = self.unary();
			left = self.add(Node::Binary { left, op, right });
		}
		left
	}

	fn unary(&mut self) -> NodeIndex {
		if let Some(op) = self.catch(&[TT::Minus, TT::Bang]) {
			let right = self.unary();
			self.add(Node::Unary { op, right })
		} else {
			self.call()
		}
	}

	fn call(&mut self) -> NodeIndex {
		let mut expr = self.primary();

		loop {
			if let Some(_) = self.catch(&[TT::LParen]) {
				let mut args = vec![];
				if self.peek().kind != TT::RParen {
					loop {
						args.push(self.expression());
						if let None = self.catch(&[TT::Comma]) {
							break;
						}
					}
				}
				let op = self.catch(&[TT::RParen]).unwrap();
				expr = self.add(Node::Call { op, expr, args })
			} else {
				break;
			}
		}

		expr
	}

	fn primary(&mut self) -> NodeIndex {
		let kind = self.peek().kind;

		match kind {
			TT::Identifier => {
				let name = self.next();
				self.add(Node::Identifier { name })
			}

			TT::True => {
				self.next();
				self.add(Node::Bool { value: true })
			}
			TT::False => {
				self.next();
				self.add(Node::Bool { value: false })
			}

			TT::Integer => {
				let value = self.next();
				self.add(Node::Integer { value })
			}
			TT::Float => {
				let value = self.next();
				self.add(Node::Float { value })
			}

			TT::LParen => {
				self.next();
				let expr = self.expression();
				if let Some(_) = self.catch(&[TT::RParen]) {
					self.add(Node::Group { expr })
				} else {
					self.add(Node::Error)
				}
			}
			TT::LBrace => {
				self.next();
				self.block(|kind| kind == TT::RBrace)
			}

			_ => {
				self.report.error(format!("unexpected token: {:?}", kind));
				self.next();
				self.add(Node::Error)
			}
		}
	}

	fn type_expression(&mut self) -> NodeIndex {
		self.type_primary()
	}

	fn type_primary(&mut self) -> NodeIndex {
		let kind = self.peek().kind;

		match kind {
			TT::Identifier => {
				let name = self.next();
				self.add(Node::Identifier { name })
			}
			_ => {
				self.report.error(format!("unexpected token: {:?}", kind));
				self.next();
				self.add(Node::Error)
			}
		}
	}

}

pub fn parse<'a>(src: &'a str, tokens: &'a TokenStream<'a>) -> Result<Ast<'a>, Report> {
	Parser::new(src, tokens).build()
}


#[cfg(test)]
mod test {
	use crate::{parse::*, token::tokenize};

	#[test]
	fn binary() {
		let src = "1 + 1";
		let tokens = tokenize(src).unwrap();
		let ast = parse(src, &tokens).unwrap();
		println!("{:#?}", ast);
	}

	#[test]
	fn call() {
		let src = "let f = fn (a) => a + a";
		let tokens = tokenize(src).unwrap();
		let ast = parse(src, &tokens).unwrap();
		println!("{:#?}", ast);
	}

}




