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

use std::fs::File;

use clap::Clap;

mod bf_memory;
mod executors;
use executors::*;

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
	/// Disables optimization passes
	#[clap(long = "disable_optimization")]
	disable_optimization_passes: bool,
	/// Prints information about recompiled operands, and memory after execution
	#[clap(short = 'v', long = "verbose")]
	verbose: bool
}
fn main() {
	let opts = Opts::parse();

	let code = {
		use std::io::Read;
		let mut code = String::new();
		File::open(opts.file_name).unwrap().read_to_string(&mut code).unwrap();
		code
	};

	macro_rules! create_static_dispatch {
		([($LeftEnum:ident, $LeftStruct:ident)], [$(($RightEnum:ident, $RightStruct:ident)), *], $left_enum:ident, $right_enum:ident) => {
			match $right_enum {
				$($RightEnum => {
					let executor = $RightStruct::new(code, $LeftStruct::new(), !opts.disable_optimization_passes, opts.verbose);
					executor.start();
					println!();
					return;
				})*
			}
		};
		
		([($LeftEnum:ident, $LeftStruct:ident), $(($LeftEnumCont:ident, $LeftStructCont:ident)), *], [$(($RightEnum:ident, $RightStruct:ident)), *], 
			$left_enum:ident, $right_enum:ident) => {
			if let $LeftEnum = $left_enum {
				create_static_dispatch!([($LeftEnum, $LeftStruct)], [$(($RightEnum, $RightStruct)), *], $left_enum, $right_enum)
			}
			create_static_dispatch!([$(($LeftEnumCont, $LeftStructCont)), *], [$(($RightEnum, $RightStruct)), *], $left_enum, $right_enum)
		}
	}
	{
		use MemoryType::*;
		use ExecutorArg::*;
		use bf_memory::*;
		use bf_opt_interpreter::BfOptInterpreter;
		use bf_interpreter::BfInterpreter;
		use bf_recompiler::BfRecompiler;
		let memory_type = &opts.memory_type;
		let struct_type = &opts.executor;
		create_static_dispatch!(
			[(DualArrayArg, BfMemoryMemSafe), (SingleArrayArg, BfMemoryMemSafeSingleArray), (UnsafeArrayArg, BfMemoryMemUnsafe)], 
			[(NewInterpreterArg, BfOptInterpreter), (OldInterpreterArg, BfInterpreter), (RecompilerArg, BfRecompiler)],
			memory_type, struct_type);
	}
}