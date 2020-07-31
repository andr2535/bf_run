/*
	This file is part of bf_run.

	bf_run is free software: you can redistribute it and/or modify
	it under the terms of the GNU General Public License as published by
	the Free Software Foundation, either version 3 of the License, or
	(at your option) any later version.

	bf_run is distributed in the hope that it will be useful,
	but WITHOUT ANY WARRANTY; without even the implied warranty of
	MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
	GNU General Public License for more details.

	You should have received a copy of the GNU General Public License
	along with bf_run.  If not, see <https://www.gnu.org/licenses/>.
*/

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