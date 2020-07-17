use crate::bf_memory::BfMemory;
use super::Executor;
use super::operations::*;

#[derive(Debug)]
pub struct BfOptInterpreter<T> {
	memory: T,
	operations: Operations,
	verbose: bool
}
impl<T: BfMemory + std::fmt::Debug> Executor<T> for BfOptInterpreter<T> {
	fn new(code: String, bf_memory: T, enable_optimizations: bool, verbose: bool) -> BfOptInterpreter<T> {
		let mut iterator = code.chars();
		let operations = Operations::conv_string_to_operations(&mut iterator);

		let mut interpreter = BfOptInterpreter{memory: bf_memory, operations, verbose};
		
		if enable_optimizations {interpreter.operations.optimize()};
		if interpreter.verbose {
			println!("Converted operations:\n{:?}", interpreter.get_ops());
		}
		interpreter
	}
	fn start(mut self) {
		let start_value = *self.memory.get_ref(0);
		let (mem_index, cur_pos_value) = BfOptInterpreter::<T>::exec_operations_vec(0, start_value, &mut self.memory, &self.operations);
		*self.memory.get_ref(mem_index) = cur_pos_value;
		if self.verbose {
			println!("\nINFO: Memory after running:\n{:?}", self.memory);
		}
	}
}
impl<T:BfMemory + std::fmt::Debug> BfOptInterpreter<T> {
	fn exec_operations_vec(mut mem_index:i32, mut cur_pos_value:u8, memory: &mut T, vec:&[Operation]) -> (i32, u8)
	{
		vec.iter().for_each(|operation| {
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
				Operation::GetInput => cur_pos_value = super::get_char(),
				Operation::PrintOutput => super::print_char(cur_pos_value)
			}
		});
		(mem_index, cur_pos_value)
	}


	pub fn get_ops(&self) -> &[Operation] {
		self.operations.as_slice()
	}
}