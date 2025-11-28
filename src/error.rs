use std::cmp::{max, min};
use std::fmt;

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub col: usize,
}

impl Span {
    pub fn new(start: usize, end: usize, line: usize, col: usize) -> Self {
        Self { start, end, line, col }
    }

    pub fn default() -> Self {
        Self { start: 0, end: 0, line: 0, col: 1 }
    }

    pub fn sum(a: Self, b: Self) -> Self {
        Self {
            start: min(a.start, b.start),
            end: max(a.end, b.end),
            line: min(a.line, b.line),
            col: min(a.col, b.col),
        }
    }
}

#[derive(Debug)]
pub struct CompilerError<'a> {
    pub message: String,
    pub span: Span,
    pub file_source: &'a String, // We need the source to print the snippet
}

impl<'a> CompilerError<'a> {
    pub fn new(message: String, span: Span, file_source: &'a String) -> Self {
        Self { message, span, file_source }
    }
}

impl<'a> fmt::Display for CompilerError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lines: Vec<&str> = self.file_source.lines().collect();
        if self.span.line == 0 || self.span.line > lines.len() {
             return write!(f, "Error: {} (at unknown location)", self.message);
        }

        let line_idx = self.span.line - 1;
        let line_content = lines[line_idx];
        
        // Calculate padding for line number
        let line_num_str = self.span.line.to_string();
        let padding = " ".repeat(line_num_str.len());

        writeln!(f, "Error: {}", self.message)?;
        writeln!(f, "  --> line:{}", self.span.line)?;
        writeln!(f, " {} |", padding)?;
        writeln!(f, " {} | {}", self.span.line, line_content)?;
        
        // Create the underline
        // self.span.col is 1-based index of the start of the token in the line
        // We need to handle tabs and such, but for now assuming simple spaces
        let col_idx = if self.span.col > 0 { self.span.col - 1 } else { 0 };
        let pointer_len = if self.span.end >= self.span.start { self.span.end - self.span.start } else { 1 };
        let pointer_len = if pointer_len == 0 { 1 } else { pointer_len };
        
        let space_prefix = " ".repeat(col_idx);
        let pointer = "^".repeat(pointer_len);
        
        writeln!(f, " {} | {}{}", padding, space_prefix, pointer)?;
        writeln!(f, " {} |", padding)?;

        Ok(())
    }
}
