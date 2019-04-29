pub trait BfMemory {
	fn get_ref(&mut self, index:i32) -> &mut u8;
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
		&mut self.array[BF_MEMORY_UNSAFE_SIZE/2 + index as usize]
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