use std::fmt;

use super::Val;

#[derive(Clone, Debug, PartialEq)]
pub enum FormatError {
    UnclosedBrace,
    UnescapedClosingBrace,
    InvalidIndex,
    InvalidAlignment,
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use FormatError::*;
        match self {
            UnclosedBrace => write!(f, "Unclosed '{{' in format string"),
            UnescapedClosingBrace => write!(f, "Unescaped '}}' in format string"),
            InvalidIndex => write!(f, "Invalid format index"),
            InvalidAlignment => write!(f, "Invalid format alignment"),
        }
    }
}

#[derive(Debug)]
enum Token {
    Text(String),
    Placeholder(Placeholder),
}

#[derive(Debug)]
struct Placeholder {
    index: usize,
    alignment: Option<i32>,
    format: Option<String>,
}

/* ============================
TOKENIZER (STATE MACHINE)
============================ */

fn tokenize(input: &str) -> Result<Vec<Token>, FormatError> {
    let chars: Vec<char> = input.chars().collect();
    let mut tokens = Vec::new();
    let mut buffer = String::new();
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            '{' => {
                // Escaped {{
                if i + 1 < chars.len() && chars[i + 1] == '{' {
                    buffer.push('{');
                    i += 2;
                } else {
                    // Flush text
                    if !buffer.is_empty() {
                        tokens.push(Token::Text(std::mem::take(&mut buffer)));
                    }

                    // Parse placeholder body
                    i += 1;
                    let start = i;

                    while i < chars.len() && chars[i] != '}' {
                        i += 1;
                    }

                    if i >= chars.len() {
                        return Err(FormatError::UnclosedBrace);
                    }

                    let content: String = chars[start..i].iter().collect();
                    let placeholder = parse_placeholder(&content)?;

                    tokens.push(Token::Placeholder(placeholder));
                    i += 1;
                }
            }

            '}' => {
                // Escaped }}
                if i + 1 < chars.len() && chars[i + 1] == '}' {
                    buffer.push('}');
                    i += 2;
                } else {
                    return Err(FormatError::UnescapedClosingBrace);
                }
            }

            c => {
                buffer.push(c);
                i += 1;
            }
        }
    }

    if !buffer.is_empty() {
        tokens.push(Token::Text(buffer));
    }

    Ok(tokens)
}

/* ============================
PLACEHOLDER PARSER
============================ */

fn parse_placeholder(s: &str) -> Result<Placeholder, FormatError> {
    // Grammar: index[,alignment][:format]

    let mut index_end = s.len();
    for (i, c) in s.char_indices() {
        if c == ',' || c == ':' {
            index_end = i;
            break;
        }
    }

    let index = s[..index_end]
        .parse::<usize>()
        .map_err(|_| FormatError::InvalidIndex)?;

    let mut alignment = None;
    let mut format = None;

    let mut rest = &s[index_end..];

    // alignment
    if rest.starts_with(',') {
        rest = &rest[1..];
        let end = rest.find(':').unwrap_or(rest.len());
        alignment = Some(
            rest[..end]
                .parse::<i32>()
                .map_err(|_| FormatError::InvalidAlignment)?,
        );
        rest = &rest[end..];
    }

    // format
    if rest.starts_with(':') {
        format = Some(rest[1..].to_string());
    }

    Ok(Placeholder {
        index,
        alignment,
        format,
    })
}

/* ============================
NUMERIC FORMAT HANDLER
============================ */

fn apply_numeric_format(value: &str, format: &str) -> Option<String> {
    if value.contains('.') || value.contains('e') || value.contains('E') {
        // Try parsing as float
        let num: f64 = value.parse().ok()?;

        if format.starts_with('N') {
            // N + digits
            let precision = format[1..].parse::<usize>().unwrap_or(2);
            // Rust thousands separator: {num:,.precision$}
            return Some(format!(
                "{num:.precision$}",
                num = num,
                precision = precision
            ));
        } else if format.starts_with('F') {
            let precision = format[1..].parse::<usize>().unwrap_or(2);
            return Some(format!(
                "{num:.precision$}",
                num = num,
                precision = precision
            ));
        }
        // other numeric formats can be added here (G, E, etc.)
    } else if format.chars().all(|c| c == '0') {
        // integer zero-padding
        let width = format.len();
        let num: i64 = value.parse().ok()?;
        return Some(format!("{:0width$}", num, width = width));
    } else if let Ok(v) = value.parse::<i64>()
        && !format.is_empty()
    {
        // integer zero-padding
        let tmp_format = format.chars().rev().collect::<Vec<char>>();
        let reversed_value = v.to_string().chars().rev().collect::<Vec<char>>();
        let mut j = 0;
        let mut result = String::new();
        for i in tmp_format {
            if i == '0' {
                if let Some(c) = reversed_value.get(j) {
                    result.push(*c);
                } else {
                    result.push('0');
                }
                j += 1;
                continue;
            } else if i == '#' {
                if let Some(c) = reversed_value.get(j) {
                    result.push(*c);
                }
                j += 1;
                continue;
            } else {
                result.push(i);
            }
        }
        return Some(result.chars().rev().collect());
    }

    None
}

/* ============================
FORMAT ENGINE
============================ */

pub fn format_ps(format: &str, args: &[String]) -> Result<String, FormatError> {
    let tokens = tokenize(format)?;
    let mut output = String::new();

    for token in tokens {
        match token {
            Token::Text(t) => output.push_str(&t),
            Token::Placeholder(p) => {
                let value = args.get(p.index).map(String::as_str).unwrap_or("");

                let mut formatted = if let Some(fmt) = &p.format {
                    if let Some(num) = apply_numeric_format(value, fmt) {
                        num
                    } else {
                        value.to_string()
                    }
                } else {
                    value.to_string()
                };

                // Alignment (PowerShell-compatible)
                if let Some(align) = p.alignment {
                    let width = align.abs() as usize;
                    if align < 0 {
                        formatted = format!("{:<width$}", formatted);
                    } else {
                        formatted = format!("{:>width$}", formatted);
                    }
                }

                output.push_str(&formatted);
            }
        }
    }

    Ok(output)
}

pub fn format_ps_vec(format_vec: Vec<Val>, args: &[Val]) -> Result<Vec<String>, FormatError> {
    let args = args
        .iter()
        .map(|v| v.cast_to_string())
        .collect::<Vec<String>>();
    let mut output = Vec::new();
    for format_elem in format_vec {
        let output_elem = format_ps(&format_elem.cast_to_string(), &args)?;
        output.push(output_elem);
    }

    Ok(output)
}
