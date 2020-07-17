use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub enum Operation {
	Mod(i8),
	Move(i32),
	Loop(Operations),
	SetValue(u8),
	GetInput,
	PrintOutput
}

#[derive(Debug, Default)]
pub struct Operations {
	operations: Vec<Operation>
}
impl Operations {
	pub fn conv_string_to_operations(iterator:&mut std::str::Chars<'_>) -> Operations {
		let mut vec = Operations::default();
		
		while let Some(character) = iterator.next() {
			match character {
				'+' => vec.push(Operation::Mod(1)),
				'-' => vec.push(Operation::Mod(-1)),
				'<' => vec.push(Operation::Move(-1)),
				'>' => vec.push(Operation::Move(1)),
				'[' => vec.push(Operation::Loop(Operations::conv_string_to_operations(iterator))),
				']' => break,
				',' => vec.push(Operation::GetInput),
				'.' => vec.push(Operation::PrintOutput),
				_ => ()
			}
		}
		vec
	}
	pub fn optimize(&mut self) {
		let new_ops = Operations::optimise_operations(self.operations.as_slice());
		*self = new_ops;
	}
	fn optimise_operations(old_ops: &[Operation]) -> Operations {
		let mut new_ops = Operations::default();
		old_ops.iter().for_each(|operation| {
			match operation {
				Operation::Mod(value) => {
					let last_mut = new_ops.last_mut();
					match last_mut {
						Some(Operation::Mod(last)) => *last = (*last).wrapping_add(*value),
						Some(Operation::SetValue(last)) => *last = (*last).wrapping_add((*value) as u8),
						_ => new_ops.push(Operation::Mod(*value))
					}
				},
				Operation::Move(value) => {
					let last_mut = new_ops.last_mut();
					match last_mut {
						Some(Operation::Move(last)) => *last += value,
						_ => new_ops.push(Operation::Move(*value))
					}
				},
				Operation::Loop(operations) => {
					if operations.len() == 1 {
						if let Some(Operation::Mod(value)) = operations.last() {
							if *value == 1 || *value == -1 {
								new_ops.push(Operation::SetValue(0));
							}
							else {
								let loop_ops = Operations::optimise_operations(operations.as_slice());
								new_ops.push(Operation::Loop(loop_ops));
							}
						}
						else {
							let loop_ops = Operations::optimise_operations(operations.as_slice());
							new_ops.push(Operation::Loop(loop_ops));
						}
					}
					else {
						let loop_ops = Operations::optimise_operations(operations.as_slice());
						new_ops.push(Operation::Loop(loop_ops));
					}
				},
				Operation::SetValue(value) => {
					let last_mut = new_ops.last_mut();
					match last_mut {
						Some(Operation::Mod(_value)) => *last_mut.unwrap() = Operation::SetValue(*value),
						Some(Operation::SetValue(_value)) => *last_mut.unwrap() = Operation::SetValue(*value),
						_ => new_ops.push(Operation::SetValue(*value))
					}
				},
				Operation::GetInput => new_ops.push(Operation::GetInput),
				Operation::PrintOutput => new_ops.push(Operation::PrintOutput)
			}
		});
		new_ops
	}
}


impl Deref for Operations {
	type Target = Vec<Operation>;
	
	fn deref(&self) -> &Vec<Operation> {
		&self.operations
	}
}

impl DerefMut for Operations {
	fn deref_mut(&mut self) -> &mut Vec<Operation> {
		&mut self.operations
	}
	
}