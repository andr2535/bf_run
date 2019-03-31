const BF_MEMORY_START_SIZE: usize = 5000;
pub trait BfMemory {
	fn get_ref(&mut self, index:i32) -> &mut u8;
}

pub struct BfMemoryMemSafe {
	negatives: Vec<u8>,
	positives: Vec<u8>
}
impl BfMemoryMemSafe {
	pub fn new() -> BfMemoryMemSafe {
		BfMemoryMemSafe{negatives:Vec::with_capacity(1000), positives:Vec::with_capacity(1000)}
	}
}
impl BfMemory for BfMemoryMemSafe {
	fn get_ref(&mut self, index: i32) -> &mut u8 {
		let (index, vec) = if index < 0 {((index * -1) as usize, &mut self.negatives)} else {(index as usize, &mut self.positives)};
		while vec.len() <= index {
			vec.push(0);
		}
		unsafe {vec.get_unchecked_mut(index)}
	}
}

pub struct BfMemoryMemSafeSingleArray {
	vector: Vec<u8>
}
impl BfMemoryMemSafeSingleArray {
	pub fn new() -> BfMemoryMemSafeSingleArray {
		BfMemoryMemSafeSingleArray{vector: vec![0u8;2]}
	}
	fn increase_memory(&mut self) {
		let old_vec_len = self.vector.len();
		let old_vec = std::mem::replace(&mut self.vector, Vec::with_capacity(old_vec_len * 2));
		
		let zero_len = old_vec_len / 2;
		for _i in 0..zero_len {self.vector.push(0);}
		for element in old_vec {self.vector.push(element);}
		for _i in 0..zero_len {self.vector.push(0);}
	}
}
impl BfMemory for BfMemoryMemSafeSingleArray {
	fn get_ref(&mut self, index: i32) -> &mut u8 {
		let vec_len = self.vector.len();
		let new_pos = index + (vec_len/2) as i32;
		if new_pos > 0 && new_pos < vec_len as i32 {
			unsafe {self.vector.get_unchecked_mut(new_pos as usize)}
		}
		else {
			self.increase_memory();
			self.get_ref(index)
		}
	}
}

pub struct BfMemoryMemUnsafe {
	array:[u8; BF_MEMORY_START_SIZE]
}
impl BfMemoryMemUnsafe {
	pub fn new() -> BfMemoryMemUnsafe {
		BfMemoryMemUnsafe{array: [0u8; BF_MEMORY_START_SIZE]}
	}
}
impl BfMemory for BfMemoryMemUnsafe {
	fn get_ref(&mut self, index: i32) -> &mut u8 {
		&mut self.array[BF_MEMORY_START_SIZE/2 + index as usize]
	}
}

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
		let mut mem_index = 0;
		let mut iterator = self.code.chars();
		let mut loop_stack = Vec::new();

		while let Some(character) = iterator.next() {
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