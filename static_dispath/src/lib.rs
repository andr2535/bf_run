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

use proc_macro::TokenStream;
use quote::{quote, TokenStreamExt};
use syn::{
	bracketed, parenthesized,
	parse::{Parse, ParseStream, Result},
	parse_macro_input, Block, Expr, Ident, Token, Type,
};

/// Parsing format like:
/// ```(type_enum_variant, type_variant)```
struct TypeMatch {
	type_enum_variant: Ident,
	type_variant:      Type,
}
impl Parse for TypeMatch {
	fn parse(input: ParseStream) -> Result<Self> {
		let parenthesized_expr;
		parenthesized!(parenthesized_expr in input);
		let type_enum_variant = parenthesized_expr.parse::<Ident>()?;
		parenthesized_expr.parse::<Token![,]>()?;
		let type_variant = parenthesized_expr.parse::<Type>()?;

		Ok(TypeMatch { type_enum_variant, type_variant })
	}
}
/// Parsing type like:
/// ```(name, actual_enum)[TypeMatch1 TypeMatch2 ...]```
struct TypeReplacement {
	name:         Ident,
	actual_enum:  Expr,
	type_matches: Vec<TypeMatch>,
}
impl Parse for TypeReplacement {
	fn parse(input: ParseStream) -> Result<Self> {
		let parenthesized_content;
		parenthesized!(parenthesized_content in input);
		let name = parenthesized_content.parse::<Ident>()?;
		parenthesized_content.parse::<Token![,]>()?;
		let actual_enum = parenthesized_content.parse::<Expr>()?;

		let bracketed_content;
		bracketed!(bracketed_content in input);
		let mut type_matches = Vec::new();
		while let Ok(type_match) = bracketed_content.parse::<TypeMatch>() {
			type_matches.push(type_match);
		}
		Ok(TypeReplacement { name, actual_enum, type_matches })
	}
}
/// Parsing type like:
/// ```ignore
/// TypeReplacement1 TypeReplacement2...
/// {
///     // Anything goes in here.
/// }
/// ```
struct StaticDispatch {
	type_replacements: Vec<TypeReplacement>,
	actions:           Block,
}
impl Parse for StaticDispatch {
	fn parse(input: ParseStream) -> Result<Self> {
		let mut type_replacements = Vec::new();
		while let Ok(type_replacement) = input.parse::<TypeReplacement>() {
			type_replacements.push(type_replacement);
		}

		let actions = input.parse::<Block>()?;
		let static_dispatch = StaticDispatch { type_replacements, actions };
		Ok(static_dispatch)
	}
}

/// This macro can be used to create a static dispatch of several different types that should be used together.
///
/// Every line consisting of
/// ```(DummyNameThatWillBeReplaced, InstantiatedEnum)[(EnumValue1, TypeToReplace1), (EnumValue2, TypeToReplace2)...]```
/// Defines a set of types, that will be chosen based on which InstantiatedEnum value is given,
/// with more lines it will generate all permutations of all types given in each line.
///
/// For each branch of these permutations you can place a block where you can place the dummyTypes.
/// The dummyTypes will be replaced with the actual types in the permutation branches.
///
/// An example of how this macro can be called is:
/// ```ignore
/// static_dispatch((FirstType, actualEnum1)[(Enum1Value1, FirstType1) (Enum1Value2, FirstType2)]
///                 (SecondType, actualEnum2)[(Enum2Value1, SecondType1) (Enum2Value2, SecondType2)]
///                 {
///                     let firstType = FirstType::new(SecondType::new());
///                 });
/// ```
/// 
/// Which expands to
/// ```ignore
/// match actualEnum1 {
///     Enum1Value1 => match actualEnum2 {
///         Enum2Value1 => {
///             let firstType = FirstType1::new(SecondType1::new());
///         }
///         Enum2Value2 => {
///             let firstType = FirstType1::new(SecondType2::new());
///         }
///     },
///     Enum1Value2 => match actualEnum2 {
///         Enum2Value1 => {
///             let firstType = FirstType2::new(SecondType1::new());
///         }
///         Enum2Value2 => {
///             let firstType = FirstType2::new(SecondType2::new());
///         }
///     },
/// }
/// ```
#[proc_macro]
pub fn static_dispatch(input: TokenStream) -> TokenStream {
	let static_dispatch = parse_macro_input!(input as StaticDispatch);

	let structure = build_recursive_match_structure(&static_dispatch.type_replacements, &static_dispatch.actions);
	TokenStream::from(quote! {
		{
			#structure
		}
	})
}

fn build_recursive_match_structure(type_replacements: &[TypeReplacement], actions: &Block) -> quote::__private::TokenStream {
	match type_replacements.len() {
		1 => {
			let type_replacement = &type_replacements[0];
			build_structure_block(type_replacement, quote! {#actions})
		},
		0 => panic!("No type replacements, invalid input to static_dispatch!"),
		_ => {
			let inner_structure = build_recursive_match_structure(&type_replacements[1..], actions);

			let type_replacement = &type_replacements[0];
			build_structure_block(type_replacement, inner_structure)
		},
	}
}
fn build_structure_block(type_replacement: &TypeReplacement, block: quote::__private::TokenStream) -> quote::__private::TokenStream {
	let name = &type_replacement.name;
	let actual_enum = &type_replacement.actual_enum;
	let mut structure = quote!();

	for type_match in &type_replacement.type_matches {
		let type_enum_variant = &type_match.type_enum_variant;
		let type_variant = &type_match.type_variant;
		let appendee = quote! {
			#type_enum_variant => #block
		};
		let appendee = replace_type_param(appendee, name, type_variant);
		structure.append_all(appendee);
	}
	return quote! {
		match #actual_enum {
			#structure
		}
	};
}
fn replace_type_param(block: quote::__private::TokenStream, to_be_replaced: &Ident, replace_with: &Type) -> quote::__private::TokenStream {
	let to_be_replaced_str = to_be_replaced.to_string();
	block
		.into_iter()
		.map(|x| match x {
			quote::__private::TokenTree::Ident(ident) if ident == to_be_replaced_str => {
				quote! {#replace_with}
			},
			quote::__private::TokenTree::Group(group) => {
				let delimiter = group.delimiter();
				let span = group.span();
				let new_stream = replace_type_param(group.stream(), to_be_replaced, replace_with);
				let mut group = quote::__private::Group::new(delimiter, new_stream);
				group.set_span(span);
				let group = quote::__private::TokenTree::Group(group);
				quote! {#group}
			},
			_ => quote! {#x},
		})
		.collect()
}
