use std::collections::{HashMap, HashSet};

pub struct IndentWriter {
    buffer: String,
    indent_level: usize,
    indent_str: &'static str,
    at_line_start: bool,
    class_name: Option<String>,
    fn_name_counts: HashMap<String, usize>,
    declared_vars: HashSet<String>,
}

impl IndentWriter {
    pub fn new(indent_str: &'static str) -> Self {
        Self {
            buffer: String::new(),
            indent_level: 0,
            indent_str,
            at_line_start: true,
            class_name: None,
            fn_name_counts: HashMap::new(),
            declared_vars: HashSet::new(),
        }
    }

    pub fn set_class_name(&mut self, name: Option<String>) {
        self.class_name = name;
    }

    pub fn class_name(&self) -> Option<&str> {
        self.class_name.as_deref()
    }

    pub fn unique_fn_name(&mut self, name: &str) -> String {
        let count = self.fn_name_counts.entry(name.to_string()).or_insert(0);
        *count += 1;
        if *count == 1 {
            name.to_string()
        } else {
            format!("{name}_{count}")
        }
    }

    pub fn declare_var(&mut self, name: &str) -> bool {
        self.declared_vars.insert(name.to_string())
    }

    pub fn clear_vars(&mut self) {
        self.declared_vars.clear();
    }

    pub fn indent(&mut self) {
        self.indent_level += 1;
    }

    pub fn dedent(&mut self) {
        self.indent_level = self.indent_level.saturating_sub(1);
    }

    pub fn write(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        if self.at_line_start {
            for _ in 0..self.indent_level {
                self.buffer.push_str(self.indent_str);
            }
            self.at_line_start = false;
        }
        self.buffer.push_str(text);
    }

    pub fn write_line(&mut self, line: &str) {
        self.write(line);
        self.newline();
    }

    pub fn newline(&mut self) {
        self.buffer.push('\n');
        self.at_line_start = true;
    }

    pub fn into_string(self) -> String {
        self.buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_writer() {
        let w = IndentWriter::new("    ");
        assert_eq!(w.into_string(), "");
    }

    #[test]
    fn simple_line() {
        let mut w = IndentWriter::new("    ");
        w.write_line("hello");
        assert_eq!(w.into_string(), "hello\n");
    }

    #[test]
    fn indented_block() {
        let mut w = IndentWriter::new("    ");
        w.write_line("if (true) {");
        w.indent();
        w.write_line("return 42;");
        w.dedent();
        w.write_line("}");
        assert_eq!(w.into_string(), "if (true) {\n    return 42;\n}\n");
    }

    #[test]
    fn nested_indent() {
        let mut w = IndentWriter::new("  ");
        w.write_line("class Foo {");
        w.indent();
        w.write_line("method() {");
        w.indent();
        w.write_line("return 1;");
        w.dedent();
        w.write_line("}");
        w.dedent();
        w.write_line("}");
        assert_eq!(
            w.into_string(),
            "class Foo {\n  method() {\n    return 1;\n  }\n}\n"
        );
    }

    #[test]
    fn write_without_newline() {
        let mut w = IndentWriter::new("    ");
        w.write("a");
        w.write(" + ");
        w.write("b");
        assert_eq!(w.into_string(), "a + b");
    }

    #[test]
    fn dedent_below_zero_saturates() {
        let mut w = IndentWriter::new("    ");
        w.dedent();
        w.write_line("no crash");
        assert_eq!(w.into_string(), "no crash\n");
    }
}
