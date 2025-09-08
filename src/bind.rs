
use std::{cell::{Ref, RefCell, RefMut}, collections::{BTreeSet, HashMap, HashSet}, hash::RandomState, vec};

use crate::{parse::{Ast, Node, NodeIndex}, token::TokenStream};


#[derive(Debug, Clone)]
enum Type {
	Var(u32),
	Int,
	Bool,
	Fn(TypeIndex, TypeIndex),
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq)]
struct TypeIndex (u32);




#[derive(Debug)]
struct Bindings<'a> {
	pool: Vec<Type>,
	count: u32,
	map: Vec<HashMap<&'a str, Option<TypeIndex>>>,
}
impl<'a> Bindings<'a> {
	fn new() -> Self {
		Self {
			pool: vec![],
			count: 0,
			map: vec![HashMap::new()],
		}
	}

	fn get(&self, index: TypeIndex) -> &Type {
		&self.pool[index.0 as usize]
	}
	fn get_mut(&mut self, index: TypeIndex) -> &mut Type {
		&mut self.pool[index.0 as usize]
	}
	fn scope_begin(&mut self) {
		self.map.push(self.map.last().cloned().unwrap());
	}
	fn scope_end(&mut self) {
		self.map.pop();
	}
}

struct Tst<'a> {
	pub tokens: &'a TokenStream<'a>,
	pub nodes: Vec<Node<'a>>,
	pub root: NodeIndex,
	pub types: Vec<Type>,
}
impl<'a> Tst<'a> {
	pub fn get(&self, node: &NodeIndex) -> &Node {
		&self.nodes[node.0 as usize]
	}
}

struct Annotate<'a> {
	ast: Ast<'a>,
	types: Vec<Option<Type>>,
}
impl<'a> Annotate<'a> {
	fn new(ast: Ast<'a>) -> Self {
		Self {
			types: vec![None; ast.nodes.len()],
			ast,
		}
	}

	fn impost(&mut self, index: NodeIndex, ) {
		
	}

	fn build(mut self) -> Tst<'a> {
		todo!()
	}

	fn annotate(&mut self, index: NodeIndex) {
		let node = self.ast.get(&index);
		
	}
}

fn annotate<'a>(ast: Ast<'a>) -> Tst<'a> {
	let mut count = 0;

	let mut types = Vec::with_capacity(ast.nodes.len());
	for _ in 0..types.len() {
		types.push(Type::Var(count));
		count += 1;
	}

	Tst {
		tokens: ast.tokens,
		nodes: ast.nodes,
		root: ast.root,
		types
	}
}


#[cfg(test)]
mod test {
    use crate::{bind::Check, parse, resolve, tokenize};

	#[test]
	fn run() {
		let src = "let x = 0";
		println!("{}", src);

		let tokens = tokenize(src).unwrap();
		println!("{:?}", tokens);

		let ast = parse(src, &tokens).unwrap();
		println!("{:#?}", ast);

		let mut bind = Check::new(src, &ast);
		bind.walk(&ast.root);
		println!("{:#?}", bind);

		panic!("complete :3")
	}
}

