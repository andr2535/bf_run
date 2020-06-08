use std::fs::File;

use clap::Clap;

pub mod bf_memory;
mod executors;
use executors::*;

#[derive(Debug)]
enum ExecutorArg {
	OldInterpreter,
	NewInterpreter,
	Recompiler
}
impl std::str::FromStr for ExecutorArg {
	type Err = ArgumentParseError;

	fn from_str(s: &str) -> Result<ExecutorArg, ArgumentParseError> {
		match s {
			"oi" => Ok(ExecutorArg::OldInterpreter),
			"ni" => Ok(ExecutorArg::NewInterpreter),
			"r"  => Ok(ExecutorArg::Recompiler),
			_ => Err(ArgumentParseError::ExecutorParseError(s.to_string()))
		}
	}
}

#[derive(Clap, Debug)]
enum MemoryType {
	UnsafeArray,
	DualArray,
	SingleArray
}
impl std::str::FromStr for MemoryType {
	type Err = ArgumentParseError;

	fn from_str(s: &str) -> Result<MemoryType, ArgumentParseError> {
		match s {
			"ua" => Ok(MemoryType::UnsafeArray),
			"da" => Ok(MemoryType::DualArray),
			"sa" => Ok(MemoryType::SingleArray),
			_ => Err(ArgumentParseError::MemoryTypeParseError(s.to_string()))
		}
	}
}

#[derive(Debug)]
enum ArgumentParseError {
	ExecutorParseError(String),
	MemoryTypeParseError(String)
}
impl std::error::Error for ArgumentParseError { }
impl std::fmt::Display for ArgumentParseError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			ArgumentParseError::ExecutorParseError(err_string) => write!(f, "Error parsing executor string '{}'", err_string),
			ArgumentParseError::MemoryTypeParseError(err_string) => write!(f, "Error parsing memory type '{}'", err_string)
		}
	}
}

#[derive(Clap, Debug)]
#[clap(name = "Brainfuck Interpreter", about = "A Brainfuck interpreter and recompiler")]
struct Opts {
	file_name: String,
	/// Old interpreter: 'oi'
	/// New interpreter: 'ni'
	/// Recompiler: 'r'
	#[clap(short = "e", long = "executor", default_value = "r")]
	executor: ExecutorArg,
	/// Unsafe array: 'ua'
	/// Single array: 'sa'
	/// Dual array: 'da'
	#[clap(short = "m", long = "memory_type", default_value = "ua")]
	memory_type: MemoryType,
	/// Disables optimization passes
	#[clap(long = "disable_optimization")]
	disable_optimization_passes: bool,
	/// Prints various information about the execution
	#[clap(short = "v", long = "verbose")]
	verbose: bool
}
fn main() {
	let opts = Opts::parse();

	let code = {
		use std::io::{Read};
		let mut code = String::new();
		File::open(opts.file_name).unwrap().read_to_string(&mut code).unwrap();
		code
	};

	match &opts.memory_type {
		MemoryType::DualArray => {
			let bf_memory = bf_memory::BfMemoryMemSafe::new();
			match &opts.executor {
				ExecutorArg::NewInterpreter => {
					let opt_interpreter = bf_opt_interpreter::BfOptInterpreter::new(code, bf_memory, !opts.disable_optimization_passes, opts.verbose);
					opt_interpreter.start();
				},
				ExecutorArg::OldInterpreter => {
					let old_interpreter = bf_interpreter::BfInterpreter::new(code, bf_memory, !opts.disable_optimization_passes, opts.verbose);
					old_interpreter.start();
				},
				ExecutorArg::Recompiler => {
					let recompiler = bf_recompiler::BfRecompiler::new(code, bf_memory, !opts.disable_optimization_passes, opts.verbose);
					recompiler.start();
				}
			}
		},
		MemoryType::SingleArray => {
			let bf_memory = bf_memory::BfMemoryMemSafeSingleArray::new();
			match &opts.executor {
				ExecutorArg::NewInterpreter => {
					let opt_interpreter = bf_opt_interpreter::BfOptInterpreter::new(code, bf_memory, !opts.disable_optimization_passes, opts.verbose);
					opt_interpreter.start();
				},
				ExecutorArg::OldInterpreter => {
					let old_interpreter = bf_interpreter::BfInterpreter::new(code, bf_memory, !opts.disable_optimization_passes, opts.verbose);
					old_interpreter.start();
				},
				ExecutorArg::Recompiler => {
					let recompiler = bf_recompiler::BfRecompiler::new(code, bf_memory, !opts.disable_optimization_passes, opts.verbose);
					recompiler.start();
				}
			}
		},
		MemoryType::UnsafeArray => {
			let bf_memory = bf_memory::BfMemoryMemUnsafe::new();
			match &opts.executor {
				ExecutorArg::NewInterpreter => {
					let opt_interpreter = bf_opt_interpreter::BfOptInterpreter::new(code, bf_memory, !opts.disable_optimization_passes, opts.verbose);
					opt_interpreter.start();
				},
				ExecutorArg::OldInterpreter => {
					let old_interpreter = bf_interpreter::BfInterpreter::new(code, bf_memory, !opts.disable_optimization_passes, opts.verbose);
					old_interpreter.start();
				},
				ExecutorArg::Recompiler => {
					let recompiler = bf_recompiler::BfRecompiler::new(code, bf_memory, !opts.disable_optimization_passes, opts.verbose);
					recompiler.start();
				}
			}
		}
	}
	println!();
}
