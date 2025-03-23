// SPDX-License-Identifier: GNU GENERAL PUBLIC LICENSE Version 3
//
// Copyleft (c) 2024 James Wong. This file is part of James Wong.
// is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the
// Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// James Wong is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with James Wong.  If not, see <https://www.gnu.org/licenses/>.
//
// IMPORTANT: Any software that fully or partially contains or uses materials
// covered by this license must also be released under the GNU GPL license.
// This includes modifications and derived works.

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{ parse_macro_input, ItemFn, LitStr, parse::Parse, parse::ParseStream, Expr };

struct LogContent {
    content: LitStr,
}

impl Parse for LogContent {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content = input.parse::<LitStr>()?;
        Ok(LogContent { content })
    }
}

fn parse_expr(input: &str) -> syn::Result<Expr> {
    syn::parse_str(input)
}

#[proc_macro_attribute]
pub fn audit_log(attr: TokenStream, item: TokenStream) -> TokenStream {
    let log_content = parse_macro_input!(attr as LogContent);
    let input = parse_macro_input!(item as ItemFn);
    let visibility = &input.vis;
    let sig = &input.sig;
    let block = &input.block;
    let content = &log_content.content.value();

    //println!("[Macro Biz Logger (like Java AOP)] - content: {:?}", content);

    let mut log_parts = Vec::new();
    let mut current_part = String::new();
    let mut in_brace = false;

    for ch in content.chars() {
        match ch {
            '{' if !in_brace => {
                if !current_part.is_empty() {
                    log_parts.push(quote! { #current_part.to_owned() });
                    current_part.clear();
                }
                in_brace = true;
            }
            '}' if in_brace => {
                if !current_part.is_empty() {
                    match parse_expr(&current_part) {
                        Ok(expr) => log_parts.push(quote! { #expr.to_string() }),
                        Err(_) => log_parts.push(quote! { #current_part.to_owned() }),
                    }
                    current_part.clear();
                }
                in_brace = false;
            }
            _ => current_part.push(ch),
        }
    }

    if !current_part.is_empty() {
        log_parts.push(quote! { #current_part.to_owned() });
    }

    let log_format = log_parts
        .iter()
        .enumerate()
        .fold(quote! {}, |acc, (i, part)| {
            if i == 0 {
                quote! { #part }
            } else {
                quote! { #acc + &#part }
            }
        });

    //println!("[Macro Biz Logger (like Java AOP)] - log_format: {:?}", log_format);

    let result =
        quote! {
        #visibility #sig {
            let log_message = #log_format;
            println!("[Macro Biz Logger (like AOP)] - Business log: {}", log_message);
            let result = (|| #block)();
            println!("[Macro Biz Logger (like AOP)] - Business operation completed.");
            result
        }
    };

    result.into()
}
