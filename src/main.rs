use std::fs::File;

mod bf_interpreter;

fn main() {
	let file = File::open("bench.bf").unwrap();
	let mut bf_int = bf_interpreter::BfInterpreter::new(file);
	bf_int.start();
	//println!("{:?}", bf_int);
}
