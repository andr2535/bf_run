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

use super::{Executor, operations::*};
use crate::bf_memory;
extern crate memmap;
use memmap::{Mmap, MmapOptions};

const PAGE_SIZE: usize = 4096;

#[derive(Debug, Default)]
pub struct RecompiledOps {
	recompiled_ops: Vec<u8>
}
impl RecompiledOps {
	pub fn push_opcodes(&mut self, opcodes: &[u8]) {
		opcodes.iter().for_each(|opcode| {
			self.recompiled_ops.push(*opcode);
		});
	}
	pub fn add_fn_call(&mut self, function_addr: usize) {
		// Pushing important registers to stack:
		self.push_opcodes(&[0x52, 0x51]); // Push rdx, push rcx.
		// Aligning Stack:
		self.push_opcodes(&[0x55]); // Push rbp. Save rbp for later restoration
		self.push_opcodes(&[0x48, 0x89, 0xe5]); // mov rsp into rbp. Backing up rsp for later restore.
		self.push_opcodes(&[0x48, 0x83, 0xe4, 0xf0]); // and rsp, -16, Align stack to 16-byte
		self.push_opcodes(&[0x48, 0x81, 0xec, 0x80, 0x00, 0x00, 0x00]); // Subtract 128, for "Red zone" per sysv64 convention
		// Placing reference to function in rax:
		let function_addr:[u8; 8] = unsafe {std::mem::transmute(function_addr)};
		self.push_opcodes(&[0x48, 0xb8]); // movabs rax
		self.push_opcodes(&function_addr); // argument for movabs rax
		// Call function
		self.push_opcodes(&[0xff, 0xd0]); // call rax
		// Restore stack.
		self.push_opcodes(&[0x48, 0x89, 0xec]); // mov rbp into rsp. Restores rsp.
		self.push_opcodes(&[0x5d]); // pop rbp. Restores rbp.
		// Restoring registers from stack:
		self.push_opcodes(&[0x59, 0x5a]); // pop rcx, pop rdx.
	}
}
impl std::ops::Deref for RecompiledOps {
	type Target = Vec<u8>;

	fn deref(&self) -> &Vec<u8> {
		&self.recompiled_ops
	}
}
impl std::ops::DerefMut for RecompiledOps {
	fn deref_mut(&mut self) -> &mut Vec<u8> {
		&mut self.recompiled_ops
	}

}

pub struct BfRecompiler<T> {
	_bf_memory: Box<T>,
	recompiled_memory: RecompiledOps,
	verbose: bool
}
impl<T: bf_memory::BfMemory + std::fmt::Debug> Executor<T> for BfRecompiler<T> {
	fn new(code: String, bf_memory: T, enable_optimizations: bool, verbose: bool) -> BfRecompiler<T> {
		// Get operations.
		let mut operations = Operations::conv_string_to_operations(code.as_ref());
		if enable_optimizations {operations.optimize();}
		if verbose {println!("Operations before recompilation to machine code:\n{:?}", operations);}

		if cfg!(target_arch = "x86_64") {
			let mut recompiled_memory = RecompiledOps::default();
			let mut bf_memory_struct = Box::new(bf_memory); // Heap allocate bf_memory.

			// First argument for get_ref, the memory.
			recompiled_memory.push_opcodes(&[0x48, 0xbf]); // movabs rdi.
			let bf_memory_addr:[u8; 8] = unsafe {std::mem::transmute(bf_memory_struct.as_mut())};
			recompiled_memory.push_opcodes(&bf_memory_addr); // bf_memory_addr as argument for movabs rdi.

			// Second argument for get_ref, the index.
			recompiled_memory.push_opcodes(&[0x48, 0xbe]); // movabs rsi.
			let index_for_bf_memory:[u8; 8] = unsafe {std::mem::transmute(0isize)};
			recompiled_memory.push_opcodes(&index_for_bf_memory); // argument for movabs rsi.

			// Get initial value of "dl"
			recompiled_memory.add_fn_call(BfRecompiler::<T>::get_ref as usize);

			// Move returned value into "dl" register, from [rax].
			recompiled_memory.push_opcodes(&[0x8a, 0x10]);
			// Set register "ecx" to zero.
			recompiled_memory.push_opcodes(&[0xb9, 0, 0, 0, 0]);

			// Perform the recompilation of the operations.
			BfRecompiler::<T>::convert_to_machine_code(&operations, unsafe {std::mem::transmute(bf_memory_struct.as_mut())}, &mut recompiled_memory);

			// Put value of "dl" back into its position in bf_memory.
			recompiled_memory.push_opcodes(&[0x88, 0x10]); // mov [rax], dl

			// Return
			recompiled_memory.push_opcodes(&[0xc3]); // return

			if verbose {
				println!("Recompiled instructions:\n{:02X?}", recompiled_memory);
			}

			BfRecompiler{_bf_memory: bf_memory_struct, recompiled_memory, verbose}
		}
		else {
			panic!("Recompiler is not implemented for this processor architecture!");
		}
	}
	fn start(self) {
		let execute_memory = self.create_exec_memory().unwrap();
		let function: fn() -> () = unsafe {std::mem::transmute(execute_memory.as_ptr())};
		function();
		if self.verbose {
			println!("\nINFO: Memory after running:\n{:?}", self._bf_memory);
		}
	}
}
impl<T:bf_memory::BfMemory + std::fmt::Debug> BfRecompiler<T> {
	extern "sysv64" fn get_ref(bf_memory: &mut T, index:i32) -> &mut u8 {
		bf_memory.get_ref(index)
	}
	extern "sysv64" fn print_u8(value:u8) {
		super::print_char(value);
	}
	extern "sysv64" fn fetch_u8() -> u8 {
		super::get_char()
	}

	/// "dl" register stores value of the currently pointed to value.
	/// "ecx" register stores the current index.
	/// "rax" register points to the last used position in memory.
	fn convert_to_machine_code(operations: &[Operation], bf_memory_addr: [u8;8], recompiled_memory: &mut RecompiledOps) {
		operations.iter().for_each(|operation| {
			match operation {
				Operation::Mod(value) => {
					// Just add value to dl.
					recompiled_memory.push_opcodes(&[0x80, 0xc2]); // Add dl
					recompiled_memory.push((*value) as u8); // Argument for add dl.
				},
				Operation::Move(move_value) => {
					// Put value of "dl" back into its position in bf_memory.
					recompiled_memory.push_opcodes(&[0x88, 0x10]); // mov [rax], dl
					
					let move_ops = T::get_move_ops(bf_memory_addr, T::get_ref as usize, *move_value);
					recompiled_memory.push_opcodes(move_ops.as_ref());

					// Move returned value into "dl" register, from [rax].
					recompiled_memory.push_opcodes(&[0x8a, 0x10]); // mov dl, [rax]
				},
				Operation::Loop(operations) => {
					let mut loop_block = RecompiledOps::default();
					BfRecompiler::<T>::convert_to_machine_code(operations, bf_memory_addr, &mut loop_block);

					let block_size = loop_block.len() as i32;

					recompiled_memory.push_opcodes(&[0x80, 0xfa]); // cmp dl
					recompiled_memory.push(0x00); // Value that dl should compare to

					// Add forward jump
					recompiled_memory.push_opcodes(&[0x0f, 0x84]); // Jump equal
					let jump_forward_length:[u8;4] = unsafe {std::mem::transmute(block_size + 5)};
					recompiled_memory.push_opcodes(&jump_forward_length);
					
					// Add loop_block
					let store = std::mem::replace(&mut *loop_block, Vec::new());
					store.into_iter().for_each(|opcode| recompiled_memory.push(opcode));
					
					// Add backwards jump
					recompiled_memory.push_opcodes(&[0xe9]); // Jump
					let jump_backwards_length:[u8;4] = unsafe{std::mem::transmute(-block_size - 5 - 9)};
					recompiled_memory.push_opcodes(&jump_backwards_length);
				},
				Operation::SetValue(value) => {
					// Set dl to value
					recompiled_memory.push(0xb2); // mov dl
					recompiled_memory.push(*value); // argument for mov dl.
				},
				Operation::GetInput => {
					recompiled_memory.push(0x50); // Push rax
					recompiled_memory.add_fn_call(BfRecompiler::<T>::fetch_u8 as usize);
					recompiled_memory.push_opcodes(&[0x88, 0xc2]); // mov dl, al
					recompiled_memory.push(0x58); // Pop rax
				},
				Operation::PrintOutput => {
					recompiled_memory.push(0x50); // Push rax
					recompiled_memory.push_opcodes(&[0x40, 0x88, 0xd7]); // mov dil, dl
					recompiled_memory.add_fn_call(BfRecompiler::<T>::print_u8 as usize);
					recompiled_memory.push(0x58); // Pop rax
				}
			}
		});
	}
	fn create_exec_memory(&self) -> Result<Mmap, BFRecompilerError> {
		let size = ((self.recompiled_memory.len() / PAGE_SIZE) + 1) * PAGE_SIZE;
		let mut mmap = MmapOptions::new().len(size).map_anon().map_err(|err| BFRecompilerError::MMapCreateError(err))?;
		let mut operand_iterator = self.recompiled_memory.iter();
		mmap.fill_with(|| *operand_iterator.next().unwrap_or(&0));
		let mmap_exec = mmap.make_exec().map_err(|err| BFRecompilerError::MMakeExecError(err))?;
		Ok(mmap_exec)
	}
}

#[derive(Debug)]
pub enum BFRecompilerError {
	MMapCreateError(std::io::Error),
	MMakeExecError(std::io::Error)
}
impl std::error::Error for BFRecompilerError { }
impl std::fmt::Display for BFRecompilerError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		use BFRecompilerError::*;
		match self {
			MMapCreateError(err) => write!(f, "Error creating jit memory: {}", err),
			MMakeExecError(err) => write!(f, "Error setting executable bit on jit memory: {}", err)
		}
	}
}