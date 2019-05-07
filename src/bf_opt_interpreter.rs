use super::bf_memory::BfMemory;

#[derive(Debug)]
pub enum Operation {
	Mod(i8),
	Move(i32),
	Loop(Vec<Operation>),
	SetValue(u8),
	GetInput,
	PrintOutput
}

#[derive(Debug)]
pub struct BfOptInterpreter<T> {
	memory: T,
	operations: Vec<Operation>
}
impl<T:BfMemory + std::fmt::Debug> BfOptInterpreter<T> {
	pub fn new(file: std::fs::File, bf_memory: T) -> BfOptInterpreter<T> {
		use std::io::{BufReader, Read};
		
		let mut string = String::new();
		let mut buf_reader = BufReader::new(file);
		buf_reader.read_to_string(&mut string).unwrap();

		let mut iterator = string.chars();
		let operations = BfOptInterpreter::<T>::conv_string_to_operations(&mut iterator);

		BfOptInterpreter{memory: bf_memory, operations: operations}
	}
	fn conv_string_to_operations(iterator:&mut std::str::Chars<'_>) -> Vec<Operation> {
		let mut vec = Vec::new();
		
		while let Some(character) = iterator.next() {
			match character {
				'+' => vec.push(Operation::Mod(1)),
				'-' => vec.push(Operation::Mod(-1)),
				'<' => vec.push(Operation::Move(-1)),
				'>' => vec.push(Operation::Move(1)),
				'[' => vec.push(Operation::Loop(BfOptInterpreter::<T>::conv_string_to_operations(iterator))),
				']' => break,
				',' => vec.push(Operation::GetInput),
				'.' => vec.push(Operation::PrintOutput),
				_ => ()
			}
		}
		vec
	}
	pub fn optimize(&mut self) {
		let new_vec = BfOptInterpreter::<T>::optimise_operations(self.operations.as_slice());
		self.operations = new_vec;
	}
	fn optimise_operations(old_vec: &[Operation]) -> Vec<Operation> {
		let mut new_vec = Vec::new();
		old_vec.into_iter().for_each(|operation| {
			match operation {
				Operation::Mod(value) => {
					let last_mut = new_vec.last_mut();
					match last_mut {
						Some(Operation::Mod(last)) => *last = (*last).wrapping_add(*value),
						Some(Operation::SetValue(last)) => *last = (*last).wrapping_add((*value) as u8),
						_ => new_vec.push(Operation::Mod(*value))
					}
				},
				Operation::Move(value) => {
					let last_mut = new_vec.last_mut();
					match last_mut {
						Some(Operation::Move(last)) => *last += value,
						_ => new_vec.push(Operation::Move(*value))
					}
				},
				Operation::Loop(operations) => {
					if operations.len() == 1 {
						if let Some(Operation::Mod(value)) = operations.last() {
							if *value == 1 || *value == -1 {
								new_vec.push(Operation::SetValue(0));
							}
							else {
								let loop_ops = BfOptInterpreter::<T>::optimise_operations(operations.as_slice());
								new_vec.push(Operation::Loop(loop_ops));
							}
						}
						else {
							let loop_ops = BfOptInterpreter::<T>::optimise_operations(operations.as_slice());
							new_vec.push(Operation::Loop(loop_ops));
						}
					}
					else {
						let loop_ops = BfOptInterpreter::<T>::optimise_operations(operations.as_slice());
						new_vec.push(Operation::Loop(loop_ops));
					}
				},
				Operation::SetValue(value) => {
					let last_mut = new_vec.last_mut();
					match last_mut {
						Some(Operation::Mod(_value)) => *last_mut.unwrap() = Operation::SetValue(*value),
						Some(Operation::SetValue(_value)) => *last_mut.unwrap() = Operation::SetValue(*value),
						_ => new_vec.push(Operation::SetValue(*value))
					}
				},
				Operation::GetInput => new_vec.push(Operation::GetInput),
				Operation::PrintOutput => new_vec.push(Operation::PrintOutput)
			}
		});
		new_vec
	}
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

	pub fn start(&mut self) {
		let start_value = *self.memory.get_ref(0);
		let (mem_index, cur_pos_value) = BfOptInterpreter::<T>::exec_operations_vec(0, start_value, &mut self.memory, &self.operations);
		*self.memory.get_ref(mem_index) = cur_pos_value;
	}

	fn exec_operations_vec(mut mem_index:i32, mut cur_pos_value:u8, memory: &mut T, vec:&Vec<Operation>) -> (i32, u8)
	{
		vec.into_iter().for_each(|operation| {
			match operation {
				Operation::Mod(value) => cur_pos_value = cur_pos_value.wrapping_add(*value as u8),
				Operation::Move(value) => {
					*memory.get_ref(mem_index) = cur_pos_value;
					mem_index -= value;
					cur_pos_value = *memory.get_ref(mem_index);
				},
				Operation::Loop(operations) => 
					while cur_pos_value != 0 {
						let (new_mem_index, new_cur_pos_value) = BfOptInterpreter::<T>::exec_operations_vec(mem_index, cur_pos_value, memory, operations);
						mem_index = new_mem_index;
						cur_pos_value = new_cur_pos_value;
					},
				Operation::SetValue(value) => cur_pos_value = *value,
				Operation::GetInput => cur_pos_value = BfOptInterpreter::<T>::get_char(),
				Operation::PrintOutput => BfOptInterpreter::<T>::print_char(cur_pos_value)
			}
		});
		(mem_index, cur_pos_value)
	}


	pub fn get_ops(&self) -> &[Operation] {
		self.operations.as_slice()
	}
}