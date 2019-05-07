use super::{bf_opt_interpreter, bf_opt_interpreter::Operation};
use super::bf_memory;
extern crate libc;

const PAGE_SIZE: usize = 4096;


struct TmpMemForJit {
	store: Vec<u8>
}
impl TmpMemForJit {
	fn new() -> TmpMemForJit {
		TmpMemForJit{store: Vec::new()}
	}
	fn push_opcodes(&mut self, opcodes: &[u8]) {
		opcodes.into_iter().for_each(|opcode| {
			self.store.push(*opcode);
		});
	}
	fn mut_vec(&mut self) -> &mut Vec<u8> {
		&mut self.store
	}
}
pub struct BfRecompiler<T> {
	_bf_memory: Box<T>,
	exec_memory: *const u8
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

		let mut tmp_exec_store = TmpMemForJit::new();
		let mut bf_memory_struct = Box::new(bf_memory); // Heap allocate bf_memory.
		
		// First argument for get_ref, the memory.
		tmp_exec_store.push_opcodes(&[0x48, 0xbf]); // movabs rdi.
		let bf_memory_addr:[u8; 8] = unsafe {std::mem::transmute(bf_memory_struct.as_mut())};
		tmp_exec_store.push_opcodes(&bf_memory_addr); // bf_memory_addr as argument for movabs rdi.
		
		// Second argument for get_ref, the index.
		tmp_exec_store.push_opcodes(&[0x48, 0xbe]); // movabs rsi.
		let index_for_bf_memory:[u8; 8] = unsafe {std::mem::transmute(0isize)};
		tmp_exec_store.push_opcodes(&index_for_bf_memory); // argument for movabs rsi.

		// Get initial value of "dl"
		BfRecompiler::<T>::add_fn_call(BfRecompiler::<T>::get_ref as usize, &mut tmp_exec_store);

		// Move returned value into "dl" register, from [rax].
		tmp_exec_store.push_opcodes(&[0x8a, 0x10]);
		// Set register "ecx" to zero.
		tmp_exec_store.push_opcodes(&[0xb9, 0, 0, 0, 0]);

		// Perform the recompilation of the operations.
		BfRecompiler::<T>::convert_to_machine_code(operations, unsafe {std::mem::transmute(bf_memory_struct.as_mut())}, &mut tmp_exec_store);
		
		// Return
		tmp_exec_store.push_opcodes(&[0xc3]); // return

		let exec_memory = BfRecompiler::<T>::create_exec_memory(tmp_exec_store);
		// exec_memory is a placeholder
		BfRecompiler{_bf_memory: bf_memory_struct, exec_memory}

	}
	fn add_fn_call(function_addr: usize, tmp_exec_store:&mut TmpMemForJit) {
		// Pushing important registers to stack:
		tmp_exec_store.push_opcodes(&[0x52, 0x51]); // Push rdx, push rcx.
		// Aligning Stack:
		tmp_exec_store.push_opcodes(&[0x55]); // Push rbp. Save rbp for later restoration
		tmp_exec_store.push_opcodes(&[0x48, 0x89, 0xe5]); // mov rsp into rbp. Backing up rsp for later restore.
		tmp_exec_store.push_opcodes(&[0x48, 0x83, 0xe4, 0xf0]); // and rsp, -16, Align stack to 16-byte
		tmp_exec_store.push_opcodes(&[0x48, 0x81, 0xec, 0x80, 0x00, 0x00, 0x00]); // Subtract 128, for "Red zone" per sysv64 convention
		// Placing reference to function in rax:
		let function_addr:[u8; 8] = unsafe {std::mem::transmute(function_addr)};
		tmp_exec_store.push_opcodes(&[0x48, 0xb8]); // movabs rax
		tmp_exec_store.push_opcodes(&function_addr); // argument for movabs rax
		// Call function
		tmp_exec_store.push_opcodes(&[0xff, 0xd0]); // call rax
		// Restore stack.
		tmp_exec_store.push_opcodes(&[0x48, 0x89, 0xec]); // mov rbp into rsp. Restores rsp.
		tmp_exec_store.push_opcodes(&[0x5d]); // pop rbp. Restores rbp.
		// Restoring registers from stack:
		tmp_exec_store.push_opcodes(&[0x59, 0x5a]); // pop rcx, pop rdx.
	}

	/// "dl" register stores the currently pointed to values.
	/// "ecx" register stores the current index.
	fn convert_to_machine_code(operations: &[Operation], bf_memory_addr: [u8;8], tmp_exec_store: &mut TmpMemForJit) {
		operations.into_iter().for_each(|operation| {
			match operation {
				Operation::Mod(value) => {
					// Just add value to dl.
					tmp_exec_store.push_opcodes(&[0x80, 0xc2]); // Add dl
					tmp_exec_store.mut_vec().push((*value) as u8); // Argument for add dl.
				},
				Operation::Move(move_value) => {
					// Put value of "dl" back into its position in bf_memory.
					// First argument for get_ref, the memory.
					tmp_exec_store.push_opcodes(&[0x48, 0xbf]); // movabs rdi.
					tmp_exec_store.push_opcodes(&bf_memory_addr); // bf_memory_addr as argument for movabs rdi.
					// Second argument for get_ref, the index.
					tmp_exec_store.push_opcodes(&[0x89, 0xce]); // mov esi, ecx.
					// Call to get the reference.
					BfRecompiler::<T>::add_fn_call(BfRecompiler::<T>::get_ref as usize, tmp_exec_store);
					// Place value back into the reference
					tmp_exec_store.push_opcodes(&[0x88, 0x10]); // mov [rax], dl
					
					// Increase the index.
					let move_value:[u8;4] = unsafe {std::mem::transmute(*move_value)};
					tmp_exec_store.push_opcodes(&[0x81, 0xc1]); // Add ecx
					tmp_exec_store.push_opcodes(&move_value); // argument for add to ecx

					// Fetch new "dl" value.
					tmp_exec_store.push_opcodes(&[0x48, 0xbf]); // movabs rdi.
					tmp_exec_store.push_opcodes(&bf_memory_addr); // bf_memory_addr as argument for movabs rdi.
					// Second argument for get_ref, the index.
					tmp_exec_store.push_opcodes(&[0x89, 0xce]); // mov esi, ecx.
					// Call to get the reference into rax.
					BfRecompiler::<T>::add_fn_call(BfRecompiler::<T>::get_ref as usize, tmp_exec_store);
					// Move returned value into "dl" register, from [rax].
					tmp_exec_store.push_opcodes(&[0x8a, 0x10]);
				},
				Operation::Loop(operations) => {
					let mut loop_block = TmpMemForJit::new();
					BfRecompiler::<T>::convert_to_machine_code(operations, bf_memory_addr, &mut loop_block);

					let block_size = loop_block.mut_vec().len() as i32;

					tmp_exec_store.push_opcodes(&[0x80, 0xfa]); // cmp dl
					tmp_exec_store.mut_vec().push(0x00); // Value that dl should compare to

					// Add forward jump
					tmp_exec_store.push_opcodes(&[0x0f, 0x84]); // Jump equal
					let jump_forward_length:[u8;4] = unsafe {std::mem::transmute(block_size + 5)};
					tmp_exec_store.push_opcodes(&jump_forward_length);
					
					// Add loop_block
					let store = std::mem::replace(loop_block.mut_vec(), Vec::new());
					store.into_iter().for_each(|opcode| tmp_exec_store.mut_vec().push(opcode));
					
					// Add backwards jump
					tmp_exec_store.push_opcodes(&[0xe9]); // Jump
					let jump_backwards_length:[u8;4] = unsafe{std::mem::transmute(-block_size - 5 - 9)};
					tmp_exec_store.push_opcodes(&jump_backwards_length);
				},
				Operation::SetValue(value) => {
					// Set dl to value
					tmp_exec_store.mut_vec().push(0xb2); // mov dl
					tmp_exec_store.mut_vec().push(*value); // argument for mov dl.
				},
				Operation::GetInput => {
					BfRecompiler::<T>::add_fn_call(BfRecompiler::<T>::fetch_u8 as usize, tmp_exec_store);
					tmp_exec_store.push_opcodes(&[0x88, 0xc2]); // mov dl, al
				},
				Operation::PrintOutput => {
					tmp_exec_store.push_opcodes(&[0x40, 0x88, 0xd7]); // mov dil, dl
					BfRecompiler::<T>::add_fn_call(BfRecompiler::<T>::print_u8 as usize, tmp_exec_store);
				}
			}
		});
	}
	fn create_exec_memory(mut tmp_mem_for_jit: TmpMemForJit) -> *const u8 {
		let tmp_mem = std::mem::replace(tmp_mem_for_jit.mut_vec(), Vec::new());
		let size = ((tmp_mem.len() / PAGE_SIZE) + 1) * PAGE_SIZE;
		let contents = unsafe {
			let mut contents: *mut libc::c_void = std::mem::uninitialized();
			libc::posix_memalign(&mut contents, PAGE_SIZE, size);
			libc::mprotect(contents, size, libc::PROT_EXEC | libc::PROT_READ | libc::PROT_WRITE);
			contents as *mut u8
		};
		tmp_mem.into_iter().enumerate().for_each(|(i, value)|{
			unsafe {*contents.offset(i as isize) = value};
		});
		contents as *const u8
	}
	pub fn start_exec(self) {
		let function: fn() -> () = unsafe {std::mem::transmute(self.exec_memory)};
		function();
	}
}
impl<T> Drop for BfRecompiler<T> {
	fn drop(&mut self) {
		unsafe {libc::free(std::mem::transmute(self.exec_memory))};
	}
}