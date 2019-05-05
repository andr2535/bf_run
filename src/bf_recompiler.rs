use super::{bf_opt_interpreter, bf_opt_interpreter::Operation};
use super::bf_memory;
extern crate libc;

const PAGE_SIZE: usize = 4096;


struct TmpMemForJit {
	store: Vec<u8>
}
impl TmpMemForJit {
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
	bf_memory: Box<T>,
	exec_memory: *const u8
}

impl<T:bf_memory::BfMemory + std::fmt::Debug> BfRecompiler<T> {
	extern "sysv64" fn get_ref(bf_memory: &mut T, index:i32) -> &mut u8 {
		let reference = bf_memory.get_ref(index);
		println!("{}", *reference);
		reference
	}
	pub fn new(file: std::fs::File, mut bf_memory: T) -> BfRecompiler<T> {
		// We use the bf_opt_int to get a list of operations, so we can use the unsafe memory without worries.
		let mut bf_opt_int = bf_opt_interpreter::BfOptInterpreter::new(file, bf_memory::BfMemoryMemUnsafe::new());
		bf_opt_int.optimize();
		let operations = bf_opt_int.get_ops();

		let mut tmp_exec_store = TmpMemForJit{store: Vec::new()};
		let mut bf_memory_struct = Box::new(bf_memory); // Heap allocate bf_memory.
		

		// Add initialization opcodes:
		tmp_exec_store.push_opcodes(&[0x55]); // Push rbp. Save rbp for later restoration
		tmp_exec_store.push_opcodes(&[0x48, 0x89, 0xe5]); // mov rsp into rbp. Backing up rsp for later restore.
		tmp_exec_store.push_opcodes(&[0x48, 0x83, 0xe4, 0xf0]); // and rsp, -16, Align stack to 16-bit
		tmp_exec_store.push_opcodes(&[0x48, 0x81, 0xec, 0x80, 0x00, 0x00, 0x00]); // Subtract 128, for "Red zone" per sysv64 convention
		
		// First argument for get_ref, the memory.
		tmp_exec_store.push_opcodes(&[0x48, 0xbf]); // movabs rdi.
		let bf_memory_addr:[u8; 8] = unsafe {std::mem::transmute(bf_memory_struct.as_mut())};
		tmp_exec_store.push_opcodes(&bf_memory_addr); // bf_memory_addr as argument for movabs rdi.
		
		// Second argument for get_ref, the index.
		tmp_exec_store.push_opcodes(&[0x48, 0xbe]); // movabs rsi.
		let index_for_bf_memory:[u8; 8] = unsafe {std::mem::transmute(0isize)};
		tmp_exec_store.push_opcodes(&index_for_bf_memory); // argument for movabs rsi.

		// Setup function reference, in rax.
		let function_addr:[u8; 8] = unsafe {std::mem::transmute(BfRecompiler::<T>::get_ref as usize)};
		tmp_exec_store.push_opcodes(&[0x48, 0xb8]); // movabs rax
		tmp_exec_store.push_opcodes(&function_addr); // argument for movabs rax

		// Call get_ref
		tmp_exec_store.push_opcodes(&[0xff, 0xd0]); // call rax

		//BfRecompiler::<T>::add_to_tmp_exec_store(operations, bf_memory, &mut tmp_exec_store);

		// Cleanup stack before return
		tmp_exec_store.push_opcodes(&[0x48, 0x89, 0xec]); // mov rbp into rsp. Restores rsp.
		tmp_exec_store.push_opcodes(&[0x5d]); // pop rbp. Restores rbp.
		
		// Return
		tmp_exec_store.push_opcodes(&[0xc3]); // return

		
		let exec_memory = BfRecompiler::<T>::create_exec_memory(tmp_exec_store);
		// exec_memory is a placeholder
		BfRecompiler{bf_memory: bf_memory_struct, exec_memory}

	}
	/// Returns an integer 
	fn add_to_tmp_exec_store(operations: &[Operation], bf_memory:*mut T, tmp_exec_store: &mut TmpMemForJit) -> usize {
		let before_size = tmp_exec_store.mut_vec().len();
		operations.into_iter().for_each(|operation| {
			match operation {
				Operation::Mod(value) => {},
				Operation::Move(value) => {},
				Operation::Loop(operations) => {},
				Operation::SetValue(value) => {},
				Operation::GetInput => {},
				Operation::PrintOutput => {}
			}
		});
		tmp_exec_store.mut_vec().len() - before_size
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
		let function: fn() -> u8 = unsafe {std::mem::transmute(self.exec_memory)};
		let ret = function();
		println!("returned was {}", ret);
	}
}