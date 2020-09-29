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
use clap::Clap;
use static_dispath::static_dispatch;

#[derive(Debug)]
enum ExecutorArg {
	OldInterpreterArg,
	NewInterpreterArg,
	RecompilerArg
}
impl std::str::FromStr for ExecutorArg {
	type Err = ArgumentParseError;

	fn from_str(s: &str) -> Result<ExecutorArg, ArgumentParseError> {
		match s {
			"oi" => Ok(ExecutorArg::OldInterpreterArg),
			"ni" => Ok(ExecutorArg::NewInterpreterArg),
			"r"  => Ok(ExecutorArg::RecompilerArg),
			_ => Err(ArgumentParseError::ExecutorParseError(s.to_string()))
		}
	}
}

#[derive(Clap, Debug)]
enum MemoryType {
	UnsafeArrayArg,
	DualArrayArg,
	SingleArrayArg
}
impl std::str::FromStr for MemoryType {
	type Err = ArgumentParseError;

	fn from_str(s: &str) -> Result<MemoryType, ArgumentParseError> {
		match s {
			"ua" => Ok(MemoryType::UnsafeArrayArg),
			"da" => Ok(MemoryType::DualArrayArg),
			"sa" => Ok(MemoryType::SingleArrayArg),
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
	#[clap(short = 'e', long = "executor", default_value = "r")]
	executor: ExecutorArg,
	/// Unsafe array: 'ua'
	/// Single array: 'sa'
	/// Dual array: 'da'
	#[clap(short = 'm', long = "memory_type", default_value = "ua")]
	memory_type: MemoryType,
	/// Sets a custom length to the internal memory of the brainfuck program.
	/// Probably only matters with "Unsafe array" memory setting.
	#[clap(long = "memory_size")]
	memory_size: Option<usize>,
	/// Disables optimization passes
	#[clap(long = "disable_optimization")]
	disable_optimization_passes: bool,
	/// Prints information about recompiled operands, and memory after execution
	#[clap(short = 'v', long = "verbose")]
	verbose: bool
}
fn main() {
	let opts = Opts::parse();
	let code = bf_run_core::read_bf_file_to_string(&opts.file_name).unwrap();
	
	{
		use MemoryType::*;
		use ExecutorArg::*;
		use bf_run_core::{bf_memory::*, executors::*};
		static_dispatch!(
			(Memory, opts.memory_type)[(DualArrayArg, BfMemoryMemSafe) (SingleArrayArg, BfMemoryMemSafeSingleArray) (UnsafeArrayArg, BfMemoryMemUnsafe)]
			(Executor, opts.executor)[(NewInterpreterArg, BfOptInterpreter) (OldInterpreterArg, BfInterpreter) (RecompilerArg, BfRecompiler)]
			{
				let executor = Executor::new(code, Memory::new(opts.memory_size), !opts.disable_optimization_passes, opts.verbose);
				executor.start();
			}
		);
	}

	println!();
}