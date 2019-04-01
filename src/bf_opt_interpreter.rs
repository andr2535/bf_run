use super::bf_memory::BfMemory;

#[derive(Debug)]
enum Operation {
	Add(u8),
	Sub(u8),
	Left(i32),
	Right(i32),
	Loop(Vec<Operation>),
	SetZero,
	GetInput,
	PrintOutput
}

pub struct BfOptInterpreter<T> {
	memory: T,
	operations: Vec<Operation>
}
impl<T:BfMemory> BfOptInterpreter<T> {
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
				'+' => vec.push(Operation::Add(1)),
				'-' => vec.push(Operation::Sub(1)),
				'<' => vec.push(Operation::Left(1)),
				'>' => vec.push(Operation::Right(1)),
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
				Operation::Add(value) => {
					if let Some(Operation::Add(last)) = new_vec.last_mut() {
						*last += value;
					}
					else {
						new_vec.push(Operation::Add(*value));
					}
				},
				Operation::Sub(value) => {
					if let Some(Operation::Sub(last)) = new_vec.last_mut() {
						*last += value;
					}
					else {
						new_vec.push(Operation::Sub(*value));
					}
				},
				Operation::Left(value) => {
					if let Some(Operation::Left(last)) = new_vec.last_mut() {
						*last += value;
					}
					else {
						new_vec.push(Operation::Left(*value));
					}
				},
				Operation::Right(value) => {
					if let Some(Operation::Right(last)) = new_vec.last_mut() {
						*last += value;
					}
					else {
						new_vec.push(Operation::Right(*value));
					}
				},
				Operation::Loop(operations) => {
					if operations.len() == 1 {
						if let Some(Operation::Sub(1)) = operations.last() {
							new_vec.push(Operation::SetZero);
						}
						else if let Some(Operation::Add(1)) = operations.last() {
							new_vec.push(Operation::SetZero);
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
				Operation::SetZero => new_vec.push(Operation::SetZero),
				Operation::GetInput => new_vec.push(Operation::GetInput),
				Operation::PrintOutput => new_vec.push(Operation::PrintOutput)
			}
		});
		new_vec
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
		BfOptInterpreter::<T>::exec_operations_vec(0, &mut self.memory, &self.operations);
	}

	fn exec_operations_vec(mut mem_index:i32, memory: &mut T, vec:&Vec<Operation>) -> i32 {
		
		vec.into_iter().for_each(|operation| {
			match operation {
				Operation::Add(value) => {
					let mem_ref = memory.get_ref(mem_index);
					*mem_ref = mem_ref.wrapping_add(*value);
				},
				Operation::Sub(value) => {
					let mem_ref = memory.get_ref(mem_index);
					*mem_ref = mem_ref.wrapping_sub(*value);
				},
				Operation::Left(value) => mem_index -= value,
				Operation::Right(value) => mem_index += value,
				Operation::Loop(operations) => {
					while *memory.get_ref(mem_index) != 0 {
						mem_index = BfOptInterpreter::<T>::exec_operations_vec(mem_index, memory, &operations);
					}
				},
				Operation::SetZero => *memory.get_ref(mem_index) = 0,
				Operation::GetInput => BfOptInterpreter::<T>::get_char(memory.get_ref(mem_index)),
				Operation::PrintOutput => BfOptInterpreter::<T>::print_char(memory.get_ref(mem_index))
			}
		});
		mem_index
	}


	pub fn print_ops(&self) {
		println!("{:?}", self.operations);
	}
}