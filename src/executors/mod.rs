use crate::bf_memory::BfMemory;

pub trait Executor<T: BfMemory + std::fmt::Debug> {
	fn new(code: String, bf_memory: T, enable_optimizations: bool, verbose: bool) -> Self;
	fn start(self);
}

pub mod bf_interpreter;
pub mod bf_opt_interpreter;
pub mod bf_recompiler;