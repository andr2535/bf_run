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

use crate::executors::bf_recompiler::RecompiledOps;

pub trait BfMemory {
	fn new(custom_size: Option<usize>) -> Self;
	fn get_ref(&mut self, index: i32) -> &mut u8;
	fn get_move_ops(bf_memory_addr: [u8; 8], get_ref_fn_addr: usize, move_value: i32) -> RecompiledOps;
	fn get_standard_move_ops(bf_memory_addr: [u8; 8], get_ref_fn_addr: usize, move_value: i32) -> RecompiledOps {
		let mut recompiled_memory = RecompiledOps::default();
		// Increase the index.
		let move_value: [u8; 4] = unsafe { std::mem::transmute(move_value) };
		recompiled_memory.push_opcodes(&[0x81, 0xc1]); // Add ecx
		recompiled_memory.push_opcodes(&move_value); // argument for add to ecx

		// Fetch reference to new "dl" value.
		recompiled_memory.push_opcodes(&[0x48, 0xbf]); // movabs rdi.
		recompiled_memory.push_opcodes(&bf_memory_addr); // bf_memory_addr as argument for movabs rdi.
		// Second argument for get_ref, the index.
		recompiled_memory.push_opcodes(&[0x89, 0xce]); // mov esi, ecx.
		recompiled_memory.add_fn_call(get_ref_fn_addr);
		recompiled_memory
	}
}

#[derive(Debug)]
pub struct BfMemoryMemSafe {
	negatives: Vec<u8>,
	positives: Vec<u8>,
}
impl BfMemory for BfMemoryMemSafe {
	fn new(custom_size: Option<usize>) -> BfMemoryMemSafe {
		let size = custom_size.map_or(0, |val| (val / 2));
		BfMemoryMemSafe { negatives: Vec::with_capacity(size), positives: Vec::with_capacity(size) }
	}

	fn get_ref(&mut self, index: i32) -> &mut u8 {
		let (index, vec) = if index < 0 {
			(index.wrapping_neg() as usize, &mut self.negatives)
		}
		else {
			(index as usize, &mut self.positives)
		};
		while vec.len() <= index {
			vec.push(0);
		}
		unsafe { vec.get_unchecked_mut(index) }
	}

	fn get_move_ops(bf_memory_addr: [u8; 8], get_ref_fn_addr: usize, move_value: i32) -> RecompiledOps {
		BfMemoryMemSafe::get_standard_move_ops(bf_memory_addr, get_ref_fn_addr, move_value)
	}
}

#[derive(Debug)]
pub struct BfMemoryMemSafeSingleArray {
	vector: Vec<u8>,
}
impl BfMemoryMemSafeSingleArray {
	#[inline(never)]
	fn increase_memory(&mut self) {
		let old_vec_len = self.vector.len();
		let old_vec = std::mem::replace(&mut self.vector, Vec::with_capacity(old_vec_len * 2));

		let zero_len = old_vec_len / 2;
		for _i in 0..zero_len {
			self.vector.push(0);
		}
		for element in old_vec {
			self.vector.push(element);
		}
		for _i in 0..zero_len {
			self.vector.push(0);
		}
	}
}
impl BfMemory for BfMemoryMemSafeSingleArray {
	fn new(custom_size: Option<usize>) -> BfMemoryMemSafeSingleArray {
		// No size below 2 can be passed, since it creates an infinite loop when trying to expand memory.
		let size = custom_size.map_or(2, |size| if size > 1 { size } else { 2 });
		BfMemoryMemSafeSingleArray { vector: vec![0u8; size] }
	}

	fn get_ref(&mut self, index: i32) -> &mut u8 {
		let vec_len = self.vector.len();
		let new_pos = index + (vec_len / 2) as i32;
		if new_pos > 0 && new_pos < vec_len as i32 {
			unsafe { self.vector.get_unchecked_mut(new_pos as usize) }
		}
		else {
			self.increase_memory();
			self.get_ref(index)
		}
	}

	fn get_move_ops(bf_memory_addr: [u8; 8], get_ref_fn_addr: usize, move_value: i32) -> RecompiledOps {
		BfMemoryMemSafe::get_standard_move_ops(bf_memory_addr, get_ref_fn_addr, move_value)
	}
}

const BF_MEMORY_UNSAFE_SIZE: usize = 65535;

#[derive(Debug)]
pub struct BfMemoryMemUnsafe {
	array: Vec<u8>,
}
impl BfMemory for BfMemoryMemUnsafe {
	fn new(custom_size: Option<usize>) -> BfMemoryMemUnsafe {
		let size = custom_size.map_or(BF_MEMORY_UNSAFE_SIZE, |val| val);
		BfMemoryMemUnsafe { array: vec![0u8; size] }
	}

	fn get_ref(&mut self, index: i32) -> &mut u8 {
		unsafe { self.array.get_unchecked_mut(((BF_MEMORY_UNSAFE_SIZE as i32) / 2 + index) as usize) }
	}

	fn get_move_ops(_bf_memory_addr: [u8; 8], _get_ref_fn_addr: usize, move_value: i32) -> RecompiledOps {
		let mut recompiled_memory = RecompiledOps::default();
		// Modify the index.
		let move_value: [u8; 4] = unsafe { std::mem::transmute(move_value) };
		recompiled_memory.push_opcodes(&[0x48, 0x8d, 0x80]); // lea rax, [rax + next argument]
		recompiled_memory.push_opcodes(&move_value); // argument for lea.

		recompiled_memory
	}
}
