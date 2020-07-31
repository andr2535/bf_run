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
use super::Executor;

#[derive(Debug)]
pub struct BfInterpreter<T> {
	memory: T,
	code: String,
	verbose: bool
}
impl<T: BfMemory + std::fmt::Debug> Executor<T> for BfInterpreter<T> {
	fn new(code: String, bf_memory: T, _enable_optimizations: bool, verbose: bool) -> BfInterpreter<T> {
		BfInterpreter{memory: bf_memory, code, verbose}
	}
	fn start(mut self) {
		let mut mem_index = 0i32;
		let mut iterator = self.code.chars();
		let mut loop_stack = Vec::new();

		while let Some(character) = iterator.next() {
			match character {
				'+' => {
					let mem_ref = self.memory.get_ref(mem_index);
					*mem_ref = mem_ref.wrapping_add(1);
				},
				'-' => {
					let mem_ref = self.memory.get_ref(mem_index);
					*mem_ref = mem_ref.wrapping_sub(1);
				},
				'<' => mem_index -= 1,
				'>' => mem_index += 1,
				',' => BfInterpreter::<T>::get_char(self.memory.get_ref(mem_index)),
				'.' => BfInterpreter::<T>::print_char(*self.memory.get_ref(mem_index)),
				'[' => {
					if *self.memory.get_ref(mem_index) != 0 {
						loop_stack.push(iterator.clone());
					}
					else {
						BfInterpreter::<T>::skip_loops(&mut iterator);
					}
				},
				']' => {
					if *self.memory.get_ref(mem_index) != 0 {
						iterator = loop_stack.last().unwrap_or_else(|| panic!("] found, while no loops had been started!")).clone();
					}
					else {
						loop_stack.pop();
					}
				}
				_ => ()
			}
		}
		if self.verbose {
			println!("\nINFO: Memory after running:\n{:?}", self.memory);
		}
	}
}
impl<T: BfMemory + std::fmt::Debug> BfInterpreter<T> {
	fn skip_loops(iterator: &mut std::str::Chars<'_>) {
		while let Some(character) = iterator.next() {
			match character {
				'[' => BfInterpreter::<T>::skip_loops(iterator),
				']' => return,
				_ => ()
			}
		}
	}

	#[inline(never)]
	fn get_char(target: &mut u8) {
		use std::io::{stdin, Read};
		let stdin = stdin();
		let mut lock = stdin.lock();
		let mut buf = [0u8; 1];
		lock.read_exact(&mut buf).unwrap();
		*target = buf[0];
	}
	#[inline(never)]
	fn print_char(source: u8) {
		print!("{}", source as char);
		use std::io;
		use io::Write;
		let mut stdout = io::stdout();
		stdout.flush().unwrap();
	}
}