use super::{bf_opt_interpreter, bf_opt_interpreter::Operation};
use super::bf_memory;
extern crate libc;

const PAGE_SIZE: usize = 4096;


pub struct RecompiledMemory {
	recompiled_ops: Vec<u8>
}
impl RecompiledMemory {
	pub fn new() -> RecompiledMemory {
		RecompiledMemory{recompiled_ops: Vec::new()}
	}
	pub fn push_opcodes(&mut self, opcodes: &[u8]) {
		opcodes.into_iter().for_each(|opcode| {
			self.recompiled_ops.push(*opcode);
		});
	}
	pub fn mut_vec(&mut self) -> &mut Vec<u8> {
		&mut self.recompiled_ops
	}
	pub fn vec(&self) -> &Vec<u8> {
		&self.recompiled_ops
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
pub struct BfRecompiler<T> {
	_bf_memory: Box<T>,
	recompiled_memory: RecompiledMemory
}

impl<T:bf_memory::BfMemory + std::fmt::Debug> BfRecompiler<T> {
	extern "sysv64" fn get_ref(bf_memory: &mut T, index:i32) -> &mut u8 {
		bf_memory.get_ref(index)
	}
	extern "sysv64" fn print_u8(value:u8) {
		bf_opt_interpreter::BfOptInterpreter::<T>::print_char(value);
	}
	extern "sysv64" fn fetch_u8() -> u8 {
		bf_opt_interpreter::BfOptInterpreter::<T>::get_char()
	}
	pub fn new(file: std::fs::File, bf_memory: T) -> BfRecompiler<T> {
		// We use the bf_opt_int to get a list of operations, so we can use the unsafe memory without worries.
		let mut bf_opt_int = bf_opt_interpreter::BfOptInterpreter::new(file, bf_memory::BfMemoryMemUnsafe::new());
		bf_opt_int.optimize();
		let operations = bf_opt_int.get_ops();

		let mut recompiled_memory = RecompiledMemory::new();
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
		BfRecompiler::<T>::convert_to_machine_code(operations, unsafe {std::mem::transmute(bf_memory_struct.as_mut())}, &mut recompiled_memory);
		
		// Return
		recompiled_memory.push_opcodes(&[0xc3]); // return

		BfRecompiler{_bf_memory: bf_memory_struct, recompiled_memory: recompiled_memory}

	}

	/// "dl" register stores value of the currently pointed to value.
	/// "ecx" register stores the current index.
	/// "rax" register points to the last used position in memory.
	fn convert_to_machine_code(operations: &[Operation], bf_memory_addr: [u8;8], recompiled_memory: &mut RecompiledMemory) {
		operations.into_iter().for_each(|operation| {
			match operation {
				Operation::Mod(value) => {
					// Just add value to dl.
					recompiled_memory.push_opcodes(&[0x80, 0xc2]); // Add dl
					recompiled_memory.mut_vec().push((*value) as u8); // Argument for add dl.
				},
				Operation::Move(move_value) => {
					// Put value of "dl" back into its position in bf_memory.
					recompiled_memory.push_opcodes(&[0x88, 0x10]); // mov [rax], dl
					
					let move_ops = T::get_move_ops(bf_memory_addr, T::get_ref as usize, *move_value);
					recompiled_memory.push_opcodes(move_ops.vec());

					// Move returned value into "dl" register, from [rax].
					recompiled_memory.push_opcodes(&[0x8a, 0x10]); // mov dl, [rax]
				},
				Operation::Loop(operations) => {
					let mut loop_block = RecompiledMemory::new();
					BfRecompiler::<T>::convert_to_machine_code(operations, bf_memory_addr, &mut loop_block);

					let block_size = loop_block.mut_vec().len() as i32;

					recompiled_memory.push_opcodes(&[0x80, 0xfa]); // cmp dl
					recompiled_memory.mut_vec().push(0x00); // Value that dl should compare to

					// Add forward jump
					recompiled_memory.push_opcodes(&[0x0f, 0x84]); // Jump equal
					let jump_forward_length:[u8;4] = unsafe {std::mem::transmute(block_size + 5)};
					recompiled_memory.push_opcodes(&jump_forward_length);
					
					// Add loop_block
					let store = std::mem::replace(loop_block.mut_vec(), Vec::new());
					store.into_iter().for_each(|opcode| recompiled_memory.mut_vec().push(opcode));
					
					// Add backwards jump
					recompiled_memory.push_opcodes(&[0xe9]); // Jump
					let jump_backwards_length:[u8;4] = unsafe{std::mem::transmute(-block_size - 5 - 9)};
					recompiled_memory.push_opcodes(&jump_backwards_length);
				},
				Operation::SetValue(value) => {
					// Set dl to value
					recompiled_memory.mut_vec().push(0xb2); // mov dl
					recompiled_memory.mut_vec().push(*value); // argument for mov dl.
				},
				Operation::GetInput => {
					recompiled_memory.mut_vec().push(0x50); // Push rax
					recompiled_memory.add_fn_call(BfRecompiler::<T>::fetch_u8 as usize);
					recompiled_memory.push_opcodes(&[0x88, 0xc2]); // mov dl, al
					recompiled_memory.mut_vec().push(0x58); // Pop rax
				},
				Operation::PrintOutput => {
					recompiled_memory.mut_vec().push(0x50); // Push rax
					recompiled_memory.push_opcodes(&[0x40, 0x88, 0xd7]); // mov dil, dl
					recompiled_memory.add_fn_call(BfRecompiler::<T>::print_u8 as usize);
					recompiled_memory.mut_vec().push(0x58); // Pop rax
				}
			}
		});
	}
	fn create_exec_memory(&self) -> *const u8 {
		let size = ((self.recompiled_memory.vec().len() / PAGE_SIZE) + 1) * PAGE_SIZE;
		let contents = unsafe {
			let mut contents: *mut libc::c_void = std::mem::uninitialized();
			libc::posix_memalign(&mut contents, PAGE_SIZE, size);
			libc::mprotect(contents, size, libc::PROT_EXEC | libc::PROT_READ | libc::PROT_WRITE);
			contents as *mut u8
		};
		self.recompiled_memory.vec().into_iter().enumerate().for_each(|(i, value)|{
			unsafe {*contents.offset(i as isize) = *value};
		});
		contents as *const u8
	}
	pub fn start_exec(self) {
		let execute_memory = self.create_exec_memory();
		let function: fn() -> () = unsafe {std::mem::transmute(execute_memory)};
		function();
		unsafe {libc::free(std::mem::transmute(execute_memory))};
	}
}