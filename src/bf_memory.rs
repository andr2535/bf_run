use crate::bf_recompiler::RecompiledMemory;

pub trait BfMemory {
	fn get_ref(&mut self, index:i32) -> &mut u8;
	fn get_move_ops(bf_memory_addr: [u8; 8], get_ref_fn_addr: usize, move_value: i32) -> RecompiledMemory;
	fn get_standard_move_ops(bf_memory_addr: [u8; 8], get_ref_fn_addr: usize, move_value: i32) -> RecompiledMemory {
		let mut recompiled_memory = RecompiledMemory::new();
		// Increase the index.
		let move_value:[u8;4] = unsafe {std::mem::transmute(move_value)};
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
	fn get_move_ops(bf_memory_addr: [u8; 8], get_ref_fn_addr: usize, move_value: i32) -> RecompiledMemory {
		BfMemoryMemSafe::get_standard_move_ops(bf_memory_addr, get_ref_fn_addr, move_value)
	}
}

#[derive(Debug)]
pub struct BfMemoryMemSafeSingleArray {
	vector: Vec<u8>
}
impl BfMemoryMemSafeSingleArray {
	pub fn new() -> BfMemoryMemSafeSingleArray {
		BfMemoryMemSafeSingleArray{vector: vec![0u8;10]}
	}
	#[inline(never)]
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
	fn get_move_ops(bf_memory_addr: [u8; 8], get_ref_fn_addr: usize, move_value: i32) -> RecompiledMemory {
		BfMemoryMemSafe::get_standard_move_ops(bf_memory_addr, get_ref_fn_addr, move_value)
	}
}

const BF_MEMORY_UNSAFE_SIZE: usize = 65535;

pub struct BfMemoryMemUnsafe {
	array:[u8; BF_MEMORY_UNSAFE_SIZE]
}
impl BfMemoryMemUnsafe {
	pub fn new() -> BfMemoryMemUnsafe {
		BfMemoryMemUnsafe{array: [0u8; BF_MEMORY_UNSAFE_SIZE]}
	}
}
impl BfMemory for BfMemoryMemUnsafe {
	fn get_ref(&mut self, index: i32) -> &mut u8 {
		unsafe {self.array.get_unchecked_mut(((BF_MEMORY_UNSAFE_SIZE as i32)/2 + index) as usize)}
	}
	fn get_move_ops(_bf_memory_addr: [u8; 8], _get_ref_fn_addr: usize, move_value: i32) -> RecompiledMemory {
		let mut recompiled_memory = RecompiledMemory::new();
		// Modify the index.
		let move_value:[u8;4] = unsafe {std::mem::transmute(move_value)};
		recompiled_memory.push_opcodes(&[0x48, 0x8d, 0x80]); // lea rax, [rax + next argument]
		recompiled_memory.push_opcodes(&move_value); // argument for lea.

		recompiled_memory
	}
}
impl std::fmt::Debug for BfMemoryMemUnsafe {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let mut string = String::with_capacity(self.array.len()*4);
		string.push('[');
		&self.array.iter().for_each(|value| {
			string.push_str(format!("{:X}, ", *value).as_ref());
		});
		if self.array.len() > 0 {
			string.pop();
			string.pop();
		}
		string.push(']');
		write!(f, "{}", string)
	}
}