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

use std::{panic, path::Path};

pub struct PanicHelper {}

impl PanicHelper {
    pub fn set_hook_default() {
        panic::set_hook(Box::new(|info| {
            if let Some(location) = info.location() {
                let file = location.file();

                // Only print src file name instead of full path.
                let file_name = Path::new(file).file_name().unwrap_or_default().to_string_lossy();

                eprintln!(
                    "Oh, Occur Panic Error panicked at {}:{}: {}",
                    file_name,
                    location.line(),
                    info.payload()
                        .downcast_ref::<&str>()
                        .unwrap_or(&"<unknown panic message>")
                );
            } else {
                println!("panic occurred but can't get location information...");
            }
        }));
    }
}
