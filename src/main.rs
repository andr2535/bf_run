use std::fs::File;

pub mod bf_memory;
mod bf_interpreter;
mod bf_opt_interpreter;
mod bf_recompiler;
use bf_recompiler::BfRecompiler;
use bf_memory::BfMemory;
fn main() {
	let file_string = {
		let mut file_string = None;
		for (i, argument) in std::env::args().enumerate() {
			if i == 0 {continue;}
			file_string = Some(argument);
		};
		file_string.unwrap()
	};

	let file = File::open(file_string).unwrap();
	let bf_memory = bf_memory::BfMemoryMemUnsafe::new();
	//let bf_memory = bf_memory::BfMemoryMemSafe::new();
	//let mut bf_memory = bf_memory::BfMemoryMemSafeSingleArray::new();
	
	let bf_recompiler = BfRecompiler::new(file, bf_memory);
	bf_recompiler.start_exec();
	/*
	let optimiser_on = true;

	if optimiser_on {
		let mut bf_int_opt = bf_opt_interpreter::BfOptInterpreter::new(file, bf_memory);
		//println!("{:?}", bf_int_opt.get_ops());
		bf_int_opt.optimize();
		//println!("\n\n");
		bf_int_opt.start();
		//println!("\n\n");
		//println!("{:?}", bf_int_opt.get_ops());
	}
	else {
		let mut bf_int = bf_interpreter::BfInterpreter::new(file, bf_memory);
		bf_int.start();
		//println!("{:?}", bf_int);
	}*/
	println!("");
}