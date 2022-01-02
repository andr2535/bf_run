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

use std::ops::{Deref, DerefMut};

#[derive(Debug, Eq, PartialEq)]
pub enum Operation {
	Mod(i8),
	Move(i32),
	Loop(Operations),
	SetValue(u8),
	GetInput,
	PrintOutput,
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct Operations {
	operations: Vec<Operation>,
}
impl Operations {
	pub fn conv_string_to_operations(code: &str) -> Operations {
		Operations::iterator_to_operations(&mut code.chars().enumerate(), None)
	}

	fn iterator_to_operations(iterator: &mut std::iter::Enumerate<std::str::Chars<'_>>, loop_start: Option<usize>) -> Operations {
		let mut vec = Operations::default();

		while let Some((index, character)) = iterator.next() {
			match character {
				'+' => vec.push(Operation::Mod(1)),
				'-' => vec.push(Operation::Mod(-1)),
				'<' => vec.push(Operation::Move(-1)),
				'>' => vec.push(Operation::Move(1)),
				'[' => vec.push(Operation::Loop(Operations::iterator_to_operations(iterator, Some(index)))),
				']' => {
					if loop_start.is_none() {
						panic!("loop terminator without matching start character at index {}", index);
					}
					return vec;
				},
				',' => vec.push(Operation::GetInput),
				'.' => vec.push(Operation::PrintOutput),
				_ => (),
			}
		}
		if let Some(loop_start) = loop_start {
			panic!("Loop started at index {} has no terminating ']' character", loop_start)
		}
		vec
	}

	pub fn optimize(&mut self) {
		loop {
			let new_ops = Operations::optimise_operations(self.operations.as_slice());
			if *self == new_ops {
				break;
			}
			*self = new_ops;
		}
	}

	fn optimise_operations(old_ops: &[Operation]) -> Operations {
		let mut new_ops = Operations::default();
		old_ops.iter().for_each(|operation| match operation {
			Operation::Mod(value) => {
				let last_mut = new_ops.last_mut();
				match last_mut {
					Some(Operation::Mod(last)) => *last = (*last).wrapping_add(*value),
					Some(Operation::SetValue(last)) => *last = (*last).wrapping_add((*value) as u8),
					_ => new_ops.push(Operation::Mod(*value)),
				}
			},
			Operation::Move(value) => {
				let last_mut = new_ops.last_mut();
				match last_mut {
					Some(Operation::Move(last)) => *last += value,
					_ => new_ops.push(Operation::Move(*value)),
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
					_ => new_ops.push(Operation::SetValue(*value)),
				}
			},
			Operation::GetInput => new_ops.push(Operation::GetInput),
			Operation::PrintOutput => new_ops.push(Operation::PrintOutput),
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
