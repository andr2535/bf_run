#[derive(Debug)]
struct BfMemory {
	negatives: Vec<u8>,
	positives: Vec<u8>
}
impl BfMemory {
	pub fn new() -> BfMemory {
		BfMemory{negatives:Vec::with_capacity(1000), positives:Vec::with_capacity(1000)}
	}
	pub fn get_ref(&mut self, index: i32) -> &mut u8 {
		let (index, vec) = if index < 0 {((index * -1) as usize, &mut self.negatives)} else {(index as usize, &mut self.positives)};
		while vec.len() <= index {
			vec.push(0);
		}
		vec.get_mut(index).unwrap()
	}
}

#[derive(Debug)]
pub struct BfInterpreter {
	memory: BfMemory,
	code: String
}
impl BfInterpreter {
	pub fn new(file: std::fs::File) -> BfInterpreter {
		use std::io::{BufReader, Read};
		
		let mut code = String::new();
		let mut buf_reader = BufReader::new(file);
		buf_reader.read_to_string(&mut code).unwrap();

		BfInterpreter{memory: BfMemory::new(), code}
	}

	fn skip_loops(iterator: &mut std::str::Chars<'_>) {
		
	}

	pub fn start(&mut self) {
		let mut mem_index = 0;
		let mut iterator = self.code.chars();
		let mut loop_stack = Vec::new();
		let mut skip_loops = 0;

		while let Some(character) = iterator.next() {
			if skip_loops == 0 {
				match character {
					'+' => {
						let mem_ref = self.memory.get_ref(mem_index);
						let new_value = mem_ref.wrapping_add(1);
						*mem_ref = new_value;
					},
					'-' => {
						let mem_ref = self.memory.get_ref(mem_index);
						let new_value = mem_ref.wrapping_sub(1);
						*mem_ref = new_value;
					},
					'<' => mem_index -= 1,
					'>' => mem_index += 1,
					',' => {unimplemented!(", is not implemented")} // Read byte from user
					'.' => print!("{}", (*self.memory.get_ref(mem_index) as u8) as char),
					'[' => {
						if *self.memory.get_ref(mem_index) != 0 {
							loop_stack.push(iterator.clone());
						}
						else {
							skip_loops += 1;
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
					other => ()
				}
			}
			else {
				match character {
					'[' => skip_loops += 1,
					']' => skip_loops -= 1,
					_ => ()
				}
			}
		}
	}

}