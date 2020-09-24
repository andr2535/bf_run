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
pub fn read_bf_file_to_string(file_name: &str) -> std::io::Result<String> {
	use std::{io::Read, fs::File};
	let mut code_u8 = Vec::new();
	File::open(file_name)?.read_to_end(&mut code_u8)?;
	Ok(String::from_utf8_lossy(code_u8.as_ref()).into())
}
pub mod bf_memory;
pub mod executors;