use anyhow::{Ok, Result};
use atty::Stream;
use console::{style, Style};
use similar::{ChangeTag, TextDiff};
use std::fmt;
use std::fmt::Write as _;
use std::io::stderr;
use std::io::Write as _;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};
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

pub fn highlight_text(text: &str, extensions: &str, theme: Option<&str>) -> Result<String> {
    // Load these once at the start of your program
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    let syntax = if let Some(s) = ps.find_syntax_by_extension(extensions) {
        s
    } else {
        ps.find_syntax_plain_text()
    };

    let mut h = HighlightLines::new(
        syntax,
        &ts.themes[theme.unwrap_or_else(|| "base16-ocean.dark")],
    );
    let mut output = String::new();
    for line in LinesWithEndings::from(text) {
        let ranges = h.highlight_line(line, &ps).unwrap();
        let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
        write!(&mut output, "{}", escaped)?;
    }
    Ok(output)
}

pub fn handle_run_err(result: Result<()>) -> Result<()> {
    match result {
        Err(e) => {
            let stderr = stderr();
            let mut stderr = stderr.lock();
            if atty::is(Stream::Stderr) {
                let s = Style::new().red();
                write!(stderr, "{:?}", s.apply_to(format!("{:?}", e)))?;
            } else {
                write!(stderr, "{:?}", e)?;
            }
        }
        _ => {}
    }

    Ok(())
}
