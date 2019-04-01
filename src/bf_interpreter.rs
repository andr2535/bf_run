use super::bf_memory::BfMemory;

pub struct BfInterpreter<T> {
	memory: T,
	code: String
}
impl<T: BfMemory> BfInterpreter<T> {
	pub fn new(file: std::fs::File, bf_memory: T) -> BfInterpreter<T> {
		use std::io::{BufReader, Read};
		
		let mut code = String::new();
		let mut buf_reader = BufReader::new(file);
		buf_reader.read_to_string(&mut code).unwrap();

		BfInterpreter{memory: bf_memory, code}
	}

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
	fn print_char(source: &u8) {
		print!("{}", *source as char);
		use std::io;
		use io::Write;
		let mut stdout = io::stdout();
		stdout.flush().unwrap();
	}
	
	pub fn start(&mut self) {
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
				'.' => BfInterpreter::<T>::print_char(self.memory.get_ref(mem_index)),
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
						iterator = loop_stack.last().unwrap().clone();
					}
					else {
						loop_stack.pop();
					}
				}
				_ => ()
			}
		}
	}
}