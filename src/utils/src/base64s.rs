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

use base64::{engine::general_purpose, Engine};

pub struct Base64Helper {}

impl Base64Helper {
    pub fn encode(input: &[u8]) -> String {
        general_purpose::STANDARD.encode(input)
    }

    pub fn decode(input: &str) -> Result<Vec<u8>, base64::DecodeError> {
        general_purpose::STANDARD.decode(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64() {
        let original = b"Hello, World!";
        let encoded = Base64Helper::encode(original);
        let decoded = Base64Helper::decode(&encoded).unwrap();
        assert_eq!(original, decoded.as_slice());
    }
}
