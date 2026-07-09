//! License templates — LICENSE file generation and SPDX identifiers.
//!
//! Chosen at `newc new` time, persisted to the project's `.newc_config.toml`,
//! and used to stamp `// SPDX-License-Identifier: <id>` into generated source files.

use std::fmt;

/// A license that can be applied to a newc project.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum License {
    Mit,
    Apache2,
    Gpl2,
    Gpl3,
    Bsd3Clause,
    Unlicense,
}

impl License {
    /// All supported licenses, in display order.
    pub fn all() -> &'static [License] {
        &[
            License::Mit,
            License::Apache2,
            License::Gpl3,
            License::Gpl2,
            License::Bsd3Clause,
            License::Unlicense,
        ]
    }

    /// SPDX license identifier, written into source files and `.newc_config.toml`.
    pub fn spdx_id(&self) -> &'static str {
        match self {
            License::Mit => "MIT",
            License::Apache2 => "Apache-2.0",
            License::Gpl2 => "GPL-2.0-only",
            License::Gpl3 => "GPL-3.0-only",
            License::Bsd3Clause => "BSD-3-Clause",
            License::Unlicense => "Unlicense",
        }
    }

    /// Human-readable name shown in the GUI license picker.
    pub fn display_name(&self) -> &'static str {
        match self {
            License::Mit => "MIT",
            License::Apache2 => "Apache License 2.0",
            License::Gpl2 => "GNU GPL v2.0",
            License::Gpl3 => "GNU GPL v3.0",
            License::Bsd3Clause => "BSD 3-Clause",
            License::Unlicense => "Unlicense",
        }
    }

    /// Parse from an SPDX identifier (case-insensitive).
    pub fn from_spdx_id(id: &str) -> Option<Self> {
        Self::all()
            .iter()
            .copied()
            .find(|l| l.spdx_id().eq_ignore_ascii_case(id))
    }

    /// Render the full `LICENSE` file body for this license.
    ///
    /// `author` and `year` fill the copyright line for licenses that need one
    /// (MIT, BSD-3-Clause); GPL and Unlicense texts are invariant.
    pub fn license_text(&self, author: &str, year: &str) -> String {
        match self {
            License::Mit => format!(
                r#"MIT License

Copyright (c) {year} {author}

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
"#
            ),
            License::Bsd3Clause => format!(
                r#"BSD 3-Clause License

Copyright (c) {year} {author}

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this
   list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright notice,
   this list of conditions and the following disclaimer in the documentation
   and/or other materials provided with the distribution.

3. Neither the name of the copyright holder nor the names of its contributors
   may be used to endorse or promote products derived from this software
   without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
"#
            ),
            License::Unlicense => {
                r#"This is free and unencumbered software released into the public domain.

Anyone is free to copy, modify, publish, use, compile, sell, or distribute
this software, either in source code form or as a compiled binary, for any
purpose, commercial or non-commercial, and by any means.

In jurisdictions that recognize copyright laws, the author or authors of this
software dedicate any and all copyright interest in the software to the
public domain. We make this dedication for the benefit of the public at large
and to the detriment of our heirs and successors. We intend this dedication
to be an overt act of relinquishment in perpetuity of all present and future
rights to this software under copyright law.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN
ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

For more information, please refer to <https://unlicense.org>
"#
                .to_string()
            }
            License::Apache2 => APACHE_2_0
                .replace("[yyyy]", year)
                .replace("[name of copyright owner]", author),
            License::Gpl2 => GPL_2_0.to_string(),
            License::Gpl3 => GPL_3_0.to_string(),
        }
    }
}

impl fmt::Display for License {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Build the one-line SPDX tag inserted as the first line of generated source files.
pub fn spdx_line(id: &str) -> String {
    format!("/* SPDX-License-Identifier: {id} */\n")
}

const APACHE_2_0: &str = include_str!("license_texts/apache-2.0.txt");
const GPL_2_0: &str = include_str!("license_texts/gpl-2.0.txt");
const GPL_3_0: &str = include_str!("license_texts/gpl-3.0.txt");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_spdx_id_round_trips() {
        for l in License::all() {
            assert_eq!(License::from_spdx_id(l.spdx_id()), Some(*l));
        }
    }

    #[test]
    fn unknown_id_returns_none() {
        assert_eq!(License::from_spdx_id("WTFPL"), None);
    }

    #[test]
    fn mit_text_includes_author_and_year() {
        let text = License::Mit.license_text("Sam", "2026");
        assert!(text.contains("Sam"));
        assert!(text.contains("2026"));
    }
}
