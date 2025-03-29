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

pub struct PanicHelper {}

impl PanicHelper {
    pub fn set_hook_default() {
        // std::panic::set_hook(Box::new(|info| {
        //     if let Some(location) = info.location() {
        //         let file_path = location.file();

        //         // Only print source relative file path instead of full path.
        //         let re = regex::Regex::new(r".*/src/(.*)").unwrap();
        //         let relative_path = if let Some(captures) = re.captures(file_path) {
        //             if let Some(path_match) = captures.get(1) {
        //                 format!("src/{}", path_match.as_str())
        //             } else {
        //                 file_path.to_string()
        //             }
        //         } else {
        //             // If there no match, return the original file path.
        //             std::path::Path::new(file_path)
        //                 .file_name()
        //                 .unwrap_or_default()
        //                 .to_string_lossy()
        //                 .to_string()
        //         };

        //         eprintln!(
        //             "Oh, Occurred Panic Error panicked at {}:{}: {}",
        //             relative_path,
        //             location.line(),
        //             info.payload().downcast_ref::<&str>().unwrap_or(&"")
        //         );

        //         // Print the full backtrace
        //         let backtrace = std::backtrace::Backtrace::capture();
        //         eprintln!("Backtrace:\n{}", backtrace);
        //     } else {
        //         println!("panic occurred but can't get location information...");
        //     }
        // }));
    }
}
