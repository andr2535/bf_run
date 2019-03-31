use std::fs::File;

mod bf_interpreter;

fn main() {
	let mut file_string = None;
	for (i, argument) in std::env::args().enumerate() {
		if i == 0 {continue;}
		file_string = Some(argument);
	};

	let file_string = if let Some(file_string) = file_string {file_string} else {panic!("Missing file path parameter!")};
	let file = File::open(file_string).unwrap();
	//let bf_memory = bf_interpreter::BfMemoryMemUnsafe::new();
	let bf_memory = bf_interpreter::BfMemoryMemSafe::new();
	//let bf_memory = bf_interpreter::BfMemoryMemSafeSingleArray::new();
	
	
	let mut bf_int = bf_interpreter::BfInterpreter::new(file, bf_memory);
	bf_int.start();
	println!("");
	//println!("{:?}", bf_int);
}
