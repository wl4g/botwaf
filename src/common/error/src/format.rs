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

use std::fmt;

use crate::ext::ErrorExt;

/// Pretty debug format for error, also prints source and backtrace.
pub struct DebugFormat<'a, E: ?Sized>(&'a E);

impl<'a, E: ?Sized> DebugFormat<'a, E> {
    /// Create a new format struct from `err`.
    pub fn new(err: &'a E) -> Self {
        Self(err)
    }
}

impl<'a, E: ErrorExt + ?Sized> fmt::Debug for DebugFormat<'a, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.", self.0)?;
        if let Some(source) = self.0.source() {
            // Source error use debug format for more verbose info.
            write!(f, " Caused by: {source:?}")?;
        }
        if let Some(location) = self.0.location_opt() {
            // Add a newline to separate causes and backtrace.
            write!(f, " at: {location}")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::any::Any;

    use snafu::prelude::*;
    use snafu::{GenerateImplicitData, Location};

    use super::*;
    use crate::ext::StackError;

    #[derive(Debug, Snafu)]
    #[snafu(display("This is a leaf error"))]
    struct Leaf;

    impl ErrorExt for Leaf {
        fn location_opt(&self) -> Option<Location> {
            None
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    impl StackError for Leaf {
        fn debug_fmt(&self, _: usize, _: &mut Vec<String>) {}

        fn next(&self) -> Option<&dyn StackError> {
            None
        }
    }

    #[derive(Debug, Snafu)]
    #[snafu(display("This is a leaf with location"))]
    struct LeafWithLocation {
        location: Location,
    }

    impl ErrorExt for LeafWithLocation {
        fn location_opt(&self) -> Option<Location> {
            None
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    impl StackError for LeafWithLocation {
        fn debug_fmt(&self, _: usize, _: &mut Vec<String>) {}

        fn next(&self) -> Option<&dyn StackError> {
            None
        }
    }

    #[derive(Debug, Snafu)]
    #[snafu(display("Internal error"))]
    struct Internal {
        #[snafu(source)]
        source: Leaf,
        location: Location,
    }

    impl ErrorExt for Internal {
        fn location_opt(&self) -> Option<Location> {
            None
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    impl StackError for Internal {
        fn debug_fmt(&self, layer: usize, buf: &mut Vec<String>) {
            buf.push(format!("{}: Internal error, at {}", layer, self.location));
            self.source.debug_fmt(layer + 1, buf);
        }

        fn next(&self) -> Option<&dyn StackError> {
            Some(&self.source)
        }
    }

    #[test]
    fn test_debug_format() {
        let err = Leaf;

        let msg = format!("{:?}", DebugFormat::new(&err));
        assert_eq!("This is a leaf error.", msg);

        let err = LeafWithLocation {
            location: Location::generate(),
        };

        // TODO(ruihang): display location here
        let msg = format!("{:?}", DebugFormat::new(&err));
        assert!(msg.starts_with("This is a leaf with location."));

        let err = Internal {
            source: Leaf,
            location: Location::generate(),
        };

        // TODO(ruihang): display location here
        let msg = format!("{:?}", DebugFormat::new(&err));
        assert!(msg.contains("Internal error. Caused by: Leaf"));
    }
}
