use crate::bf_memory::BfMemory;


pub fn get_char() -> u8 {
	use std::io::{stdin, Read};
	let stdin = stdin();
	let mut lock = stdin.lock();
	let mut buf = [0u8; 1];
	lock.read_exact(&mut buf).unwrap();
	buf[0]
}
pub fn print_char(source: u8) {
	print!("{}", source as char);
	use std::io;
	use io::Write;
	let mut stdout = io::stdout();
	stdout.flush().unwrap();
}
pub trait Executor<T: BfMemory + std::fmt::Debug> {
	fn new(code: String, bf_memory: T, enable_optimizations: bool, verbose: bool) -> Self;
	fn start(self);
}

pub mod operations;
pub mod bf_interpreter;
pub mod bf_opt_interpreter;
pub mod bf_recompiler;