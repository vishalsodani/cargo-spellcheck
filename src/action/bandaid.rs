//! A mistake bandaid.
//!
//! Covers the mistake with a suggested replacement, as
//! picked by the user.

use crate::span::Span;
use crate::suggestion::Suggestion;
use anyhow::{bail, Error, Result};
use log::trace;
use std::convert::TryFrom;

/// A choosen sugestion for a certain span
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BandAid {
    /// a span, where the first line has index 1, columns are base 0
    pub span: Span,
    /// replacement text for the given span
    pub replacement: String,
}

impl BandAid {
    /// Create a new bandaid from a span and the content to replace the
    /// spans current content.
    pub fn new(replacement: &str, span: &Span) -> Self {
        trace!(
            "span of doc comment: ({},{})..({},{})",
            span.start.line,
            span.start.column,
            span.end.line,
            span.end.column
        );

        Self {
            span: *span,
            replacement: replacement.to_owned(),
        }
    }
}

impl<'s> TryFrom<(&Suggestion<'s>, usize)> for BandAid {
    type Error = Error;
    fn try_from((suggestion, pick_idx): (&Suggestion<'s>, usize)) -> Result<Self> {
        let literal_file_span = suggestion.span;
        trace!(
            "proc_macro literal span of doc comment: ({},{})..({},{})",
            literal_file_span.start.line,
            literal_file_span.start.column,
            literal_file_span.end.line,
            literal_file_span.end.column
        );

        if let Some(replacement) = suggestion.replacements.iter().nth(pick_idx) {
            Ok(Self::new(replacement.as_str(), &suggestion.span))
        } else {
            bail!("Does not contain any replacements")
        }
    }
}

impl<'s> TryFrom<(Suggestion<'s>, usize)> for BandAid {
    type Error = Error;
    fn try_from((suggestion, pick_idx): (Suggestion<'s>, usize)) -> Result<Self> {
        Self::try_from((&suggestion, pick_idx))
    }
}

impl From<(String, Span)> for BandAid {
    fn from((replacement, span): (String, Span)) -> Self {
        Self { span, replacement }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::util::load_span_from;
    use crate::{LineColumn, Span};
    use anyhow::anyhow;
    use std::path::Path;

    /// Extract span from file as String
    /// Helpful to validate bandaids against what's actually in the file
    #[allow(unused)]
    pub(crate) fn load_span_from_file(path: impl AsRef<Path>, span: Span) -> Result<String> {
        let path = path.as_ref();
        let path = path
            .canonicalize()
            .map_err(|e| anyhow!("Failed to canonicalize {}", path.display()).context(e))?;

        let ro = std::fs::OpenOptions::new()
            .read(true)
            .open(&path)
            .map_err(|e| anyhow!("Failed to open {}", path.display()).context(e))?;

        let mut reader = std::io::BufReader::new(ro);

        load_span_from(reader, span)
    }

    #[test]
    fn span_helper_integrity() {
        const SOURCE: &'static str = r#"0
abcde
f
g
hijk
l
"#;

        struct TestSet {
            span: Span,
            expected: &'static str,
        }

        const SETS: &[TestSet] = &[
            TestSet {
                span: Span {
                    start: LineColumn {
                        line: 1usize,
                        column: 0,
                    },
                    end: LineColumn {
                        line: 1usize,
                        column: 0,
                    },
                },
                expected: "0",
            },
            TestSet {
                span: Span {
                    start: LineColumn {
                        line: 2usize,
                        column: 2,
                    },
                    end: LineColumn {
                        line: 2usize,
                        column: 4,
                    },
                },
                expected: "cde",
            },
            TestSet {
                span: Span {
                    start: LineColumn {
                        line: 5usize,
                        column: 0,
                    },
                    end: LineColumn {
                        line: 5usize,
                        column: 1,
                    },
                },
                expected: "hi",
            },
        ];

        for item in SETS {
            assert_eq!(
                load_span_from(SOURCE.as_bytes(), item.span).unwrap(),
                item.expected.to_string()
            );
        }
    }

    #[test]
    fn try_from_string_works() {
        const TEST: &str = include_str!("../../demo/src/main.rs");

        const EXPECTED: &[Span] = &[
            Span {
                start: LineColumn { line: 1, column: 4 },
                end: LineColumn { line: 1, column: 7 },
            },
            Span {
                start: LineColumn { line: 1, column: 9 },
                end: LineColumn { line: 1, column: 9 },
            },
            Span {
                start: LineColumn {
                    line: 1,
                    column: 11,
                },
                end: LineColumn {
                    line: 1,
                    column: 13,
                },
            },
            Span {
                start: LineColumn {
                    line: 1,
                    column: 15,
                },
                end: LineColumn {
                    line: 1,
                    column: 22,
                },
            },
            Span {
                start: LineColumn {
                    line: 1,
                    column: 24,
                },
                end: LineColumn {
                    line: 1,
                    column: 31,
                },
            },
        ];

        crate::checker::tests::extraction_test_body(TEST, EXPECTED);
    }

    #[test]
    fn try_from_raw_string_works() {
        const TEST: &str = include_str!("../../demo/src/lib.rs");
        let fn_with_doc = TEST
            .lines()
            .skip(18)
            .fold(String::new(), |acc, line| acc + line);

        const EXPECTED: &[Span] = &[
            Span {
                start: LineColumn {
                    line: 1,
                    column: 11,
                },
                end: LineColumn {
                    line: 1,
                    column: 14,
                },
            },
            Span {
                start: LineColumn {
                    line: 1,
                    column: 16,
                },
                end: LineColumn {
                    line: 1,
                    column: 17,
                },
            },
            Span {
                start: LineColumn {
                    line: 1,
                    column: 19,
                },
                end: LineColumn {
                    line: 1,
                    column: 21,
                },
            },
            Span {
                start: LineColumn {
                    line: 1,
                    column: 23,
                },
                end: LineColumn {
                    line: 1,
                    column: 26,
                },
            },
            Span {
                start: LineColumn {
                    line: 1,
                    column: 28,
                },
                end: LineColumn {
                    line: 1,
                    column: 32,
                },
            },
            Span {
                start: LineColumn {
                    line: 1,
                    column: 35,
                },
                end: LineColumn {
                    line: 1,
                    column: 38,
                },
            },
            Span {
                start: LineColumn {
                    line: 1,
                    column: 40,
                },
                end: LineColumn {
                    line: 1,
                    column: 43,
                },
            },
            Span {
                start: LineColumn {
                    line: 1,
                    column: 45,
                },
                end: LineColumn {
                    line: 1,
                    column: 47,
                },
            },
            Span {
                start: LineColumn {
                    line: 1,
                    column: 49,
                },
                end: LineColumn {
                    line: 1,
                    column: 53,
                },
            },
            Span {
                start: LineColumn {
                    line: 1,
                    column: 55,
                },
                end: LineColumn {
                    line: 1,
                    column: 57,
                },
            },
            Span {
                start: LineColumn {
                    line: 1,
                    column: 59,
                },
                end: LineColumn {
                    line: 1,
                    column: 61,
                },
            },
        ];

        crate::checker::tests::extraction_test_body(fn_with_doc.as_str(), EXPECTED);
    }
}
