use std::cmp::{max, min, PartialEq};
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

#[derive(Debug, PartialEq)]
pub enum CompilerErrorType {
    Error,
    Warning,
    Note,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogStyle {
    Default,
    Plain,
    Short,
    Json,
}

#[derive(Debug)]
pub struct CompilerError<'a> {
    pub ty: CompilerErrorType,
    pub message: String,
    pub span: Span,
    pub file_source: &'a String,
    pub secondary_spans: Vec<(Span, String)>,
}

impl<'a> CompilerError<'a> {
    pub fn new(message: String, span: Span, file_source: &'a String) -> Self {
        Self { ty: CompilerErrorType::Error, message, span, file_source, secondary_spans: vec![] }
    }
    
    pub fn warning(message: String, span: Span, file_source: &'a String) -> Self {
        Self { ty: CompilerErrorType::Warning, message, span, file_source, secondary_spans: vec![] }
    }
    
    pub fn note(message: String, span: Span, file_source: &'a String) -> Self {
        Self { ty: CompilerErrorType::Note, message, span, file_source, secondary_spans: vec![] }
    }

    pub fn with_related_span(mut self, span: Span, label: String) -> Self {
        self.secondary_spans.push((span, label));
        self
    }

    pub fn format_type(type_name: &str) -> String {
        format!("`{}`", type_name)
    }

    pub fn render(&self, style: LogStyle) -> String {
        match style {
            LogStyle::Default => self.render_pretty(true),
            LogStyle::Plain => self.render_pretty(false),
            LogStyle::Short => self.render_short(),
            LogStyle::Json => self.render_json(),
        }
    }

    fn render_short(&self) -> String {
        let type_str = match self.ty {
            CompilerErrorType::Error => "error",
            CompilerErrorType::Warning => "warning",
            CompilerErrorType::Note => "note",
        };
        let mut out = format!("{}:{}: {}: {}", self.span.line, self.span.col, type_str, self.message);
        for (span, label) in &self.secondary_spans {
             out.push_str(&format!("\n{}:{}: note: {}", span.line, span.col, label));
        }
        out
    }

    fn render_json(&self) -> String {
        let escaped_msg = self.message.replace("\"", "\\\"").replace("\n", "\\n");
        let type_str = match self.ty {
            CompilerErrorType::Error => "error",
            CompilerErrorType::Warning => "warning",
            CompilerErrorType::Note => "note",
        };
        
        let mut secondary_json = String::new();
        if !self.secondary_spans.is_empty() {
            secondary_json.push_str(", \"related\": [");
            for (i, (span, label)) in self.secondary_spans.iter().enumerate() {
                if i > 0 { secondary_json.push_str(", "); }
                let escaped_label = label.replace("\"", "\\\"").replace("\n", "\\n");
                secondary_json.push_str(&format!(
                    r#"{{"line": {}, "col": {}, "message": "{}"}}"#,
                    span.line, span.col, escaped_label
                ));
            }
            secondary_json.push_str("]");
        }

        format!(
            r#"{{"type": "{}", "line": {}, "col": {}, "message": "{}"{}}}"#,
            type_str, self.span.line, self.span.col, escaped_msg, secondary_json
        )
    }

    fn render_snippet(&self, out: &mut String, span: &Span, label: Option<&str>, use_colors: bool, is_primary: bool) {
        let cyan = "\x1b[36m";
        let reset = "\x1b[0m";
        let bold = "\x1b[1m";
        let blue = "\x1b[34m"; // For notes/secondary

        let lines: Vec<&str> = self.file_source.lines().collect();
        if span.line == 0 || span.line > lines.len() {
             out.push_str("  (at unknown location)\n");
             return;
        }

        let line_idx = span.line - 1;
        let line_content = lines[line_idx];
        
        let line_num_str = span.line.to_string();
        let padding = " ".repeat(line_num_str.len());
        
        // Location header
        if use_colors {
            if let Some(lbl) = label {
                out.push_str(&format!("  {}-->{} {}:{}: {}\n", cyan, reset, span.line, span.col, lbl));
            } else {
                out.push_str(&format!("  {}-->{} line:{}\n", cyan, reset, span.line));
            }
            out.push_str(&format!(" {} {}|\n", padding, cyan));
            out.push_str(&format!(" {} {}|{} {}\n", span.line, cyan, reset, line_content));
        } else {
            if let Some(lbl) = label {
                out.push_str(&format!("  --> {}:{}: {}\n", span.line, span.col, lbl));
            } else {
                out.push_str(&format!("  --> line:{}\n", span.line));
            }
            out.push_str(&format!(" {} |\n", padding));
            out.push_str(&format!(" {} | {}\n", span.line, line_content));
        }
        
        // Underline
        let col_idx = if span.col > 0 { span.col - 1 } else { 0 };
        let col_idx = min(col_idx, line_content.len());
        
        let pointer_len = if span.end >= span.start { span.end - span.start } else { 1 };
        let pointer_len = if pointer_len == 0 { 1 } else { pointer_len };
        
        let space_prefix = " ".repeat(col_idx);
        let pointer_char = if is_primary { "^" } else { "-" };
        let pointer = pointer_char.repeat(pointer_len);
        
        if use_colors {
             let color = if is_primary { bold } else { blue };
             out.push_str(&format!(" {} {}|{} {}{}{}\n", padding, cyan, reset, space_prefix, color, pointer));
             out.push_str(&format!(" {} {}|{}\n", padding, cyan, reset));
        } else {
             out.push_str(&format!(" {} | {}{}\n", padding, space_prefix, pointer));
             out.push_str(&format!(" {} |\n", padding));
        }
    }

    fn render_pretty(&self, use_colors: bool) -> String {
        let (color_code, type_str) = match self.ty {
            CompilerErrorType::Error => ("\x1b[31m", "Error"),
            CompilerErrorType::Warning => ("\x1b[33m", "Warning"),
            CompilerErrorType::Note => ("\x1b[34m", "Note"),
        };
        let reset = "\x1b[0m";
        let bold = "\x1b[1m";

        let mut out = String::new();

        // Main Header
        if use_colors {
            out.push_str(&format!("{}{}{}:{} {}\n", bold, color_code, type_str, reset, self.message));
        } else {
            out.push_str(&format!("{}: {}\n", type_str, self.message));
        }

        // Primary Span
        self.render_snippet(&mut out, &self.span, None, use_colors, true);

        // Secondary Spans
        for (span, label) in &self.secondary_spans {
            self.render_snippet(&mut out, span, Some(label), use_colors, false);
        }

        out
    }
}

impl<'a> fmt::Display for CompilerError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.render(LogStyle::Default))
    }
}
