use anyhow::{Ok, Result};
use console::{style, Style};
use similar::{ChangeTag, TextDiff};
use std::fmt;
use std::fmt::Write;
struct Line(Option<usize>);

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            None => write!(f, "    "),
            Some(idx) => write!(f, "{:<4}", idx + 1),
        }
    }
}

pub fn diff_text_to_terminal_inline(text1: &str, text2: &str) -> Result<String> {
    let mut diff_str = String::new();
    let diff = TextDiff::from_lines(text1, text2);

    for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
        if idx > 0 {
            write!(diff_str, "{:-^1$}", "-", 80)?;
        }
        for op in group {
            for change in diff.iter_inline_changes(op) {
                let (sign, s) = match change.tag() {
                    ChangeTag::Delete => ("-", Style::new().red()),
                    ChangeTag::Insert => ("+", Style::new().green()),
                    ChangeTag::Equal => (" ", Style::new().dim()),
                };
                write!(
                    diff_str,
                    "{}{} |{}",
                    style(Line(change.old_index())).dim(),
                    style(Line(change.new_index())).dim(),
                    s.apply_to(sign).bold()
                )?;
                for (emphasized, value) in change.iter_strings_lossy() {
                    if emphasized {
                        write!(diff_str, "{}", s.apply_to(value).underlined().on_black())?;
                    } else {
                        write!(diff_str, "{}", s.apply_to(value))?;
                    }
                }
                if change.missing_newline() {
                    println!();
                }
            }
        }
    }
    Ok(diff_str)
}
