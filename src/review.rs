//! Review helpers: language detection, line-based code splitting and prompt builders.

use std::path::Path;

/// Maximum number of lines in one code piece.
pub const MAX_PIECE_LINES: usize = 60;

/// A slice of a source file. `start` and `end` are 1-based, inclusive line numbers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodePiece {
    pub start: usize,
    pub end: usize,
    pub text: String,
}

/// Returns the language name for a file path, based on the file extension.
pub fn detect_language(path: &str) -> Option<&'static str> {
    let ext = Path::new(path).extension()?.to_str()?.to_ascii_lowercase();
    Some(match ext.as_str() {
        "py" => "Python",
        "rs" => "Rust",
        "js" | "mjs" | "cjs" => "JavaScript",
        "ts" | "tsx" => "TypeScript",
        "jsx" => "JavaScript (JSX)",
        "java" => "Java",
        "kt" => "Kotlin",
        "go" => "Go",
        "c" | "h" => "C",
        "cpp" | "cc" | "cxx" | "hpp" => "C++",
        "cs" => "C#",
        "php" => "PHP",
        "rb" => "Ruby",
        "swift" => "Swift",
        "sh" | "bash" => "Shell",
        "sql" => "SQL",
        "ps1" => "PowerShell",
        _ => return None,
    })
}

/// Splits code into pieces of at most `max_lines` lines. Never splits a line.
pub fn split_code(code: &str, max_lines: usize) -> Vec<CodePiece> {
    let max_lines = max_lines.max(1);
    let lines: Vec<&str> = code.lines().collect();
    lines
        .chunks(max_lines)
        .enumerate()
        .filter(|(_, group)| group.iter().any(|l| !l.trim().is_empty()))
        .map(|(i, group)| {
            let start = i * max_lines + 1;
            CodePiece {
                start,
                end: start + group.len() - 1,
                text: group.join("\n"),
            }
        })
        .collect()
}

/// Builds the prompt for one code piece. The code has no line-number prefixes,
/// so the model cannot copy them into its fix. It gets the first line number instead.
pub fn build_prompt(language: &str, file: &str, piece: &CodePiece) -> String {
    format!(
        "Review this {language} code for security problems.
File: {file}
The first line of the code below is line {}. The last line is line {}.

```
{}
```

Answer in the required sections.",
        piece.start, piece.end, piece.text
    )
}

/// Builds the prompt for a plain secure coding question.
pub fn build_question_prompt(question: &str) -> String {
    format!("Secure coding question:\n{question}\n\nAnswer in the required sections.")
}

/// Makes a file-name-safe slug, for example `src/login.py` becomes `login-py`.
pub fn review_slug(path: &str) -> String {
    let name = Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path);
    let mut out = String::new();
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    let out = out.trim_matches('-').to_string();
    if out.is_empty() {
        "file".to_string()
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_known_languages() {
        assert_eq!(detect_language("src/login.py"), Some("Python"));
        assert_eq!(detect_language("a/B.RS"), Some("Rust"));
        assert_eq!(detect_language("x.tsx"), Some("TypeScript"));
        assert_eq!(detect_language("q.sql"), Some("SQL"));
    }

    #[test]
    fn unknown_or_missing_extension_gives_none() {
        assert_eq!(detect_language("notes.xyz"), None);
        assert_eq!(detect_language("Makefile"), None);
    }

    #[test]
    fn split_empty_gives_no_pieces() {
        assert!(split_code("", 60).is_empty());
        assert!(split_code("\n\n  \n", 60).is_empty());
    }

    #[test]
    fn split_short_code_gives_one_piece() {
        let p = split_code("a\nb\nc", 60);
        assert_eq!(p.len(), 1);
        assert_eq!((p[0].start, p[0].end), (1, 3));
        assert_eq!(p[0].text, "a\nb\nc");
    }

    #[test]
    fn split_respects_max_lines_and_keeps_all_lines() {
        let code: String = (1..=130).map(|i| format!("line{i}\n")).collect();
        let p = split_code(&code, 60);
        assert_eq!(p.len(), 3);
        assert_eq!((p[0].start, p[0].end), (1, 60));
        assert_eq!((p[1].start, p[1].end), (61, 120));
        assert_eq!((p[2].start, p[2].end), (121, 130));
        let rebuilt: Vec<&str> = p.iter().flat_map(|x| x.text.lines()).collect();
        assert_eq!(rebuilt.len(), 130);
        for piece in &p {
            assert!(piece.text.lines().count() <= 60);
        }
    }

    #[test]
    fn split_zero_max_does_not_panic() {
        assert_eq!(split_code("a\nb", 0).len(), 2);
    }

    #[test]
    fn prompt_has_language_file_and_real_line_numbers() {
        let piece = CodePiece {
            start: 61,
            end: 62,
            text: "x = 1
y = 2"
                .into(),
        };
        let s = build_prompt("Python", "a.py", &piece);
        assert!(s.contains("Python"));
        assert!(s.contains("File: a.py"));
        assert!(s.contains("line 61"));
        assert!(s.contains("line 62"));
        assert!(s.contains(
            "```
x = 1
y = 2
```"
        ));
    }

    #[test]
    fn question_prompt_contains_question() {
        assert!(build_question_prompt("How to hash?").contains("How to hash?"));
    }

    #[test]
    fn slug_is_safe() {
        assert_eq!(review_slug("src/login.py"), "login-py");
        assert_eq!(review_slug("..\\My File!.JS"), "my-file-js");
        assert_eq!(review_slug("///"), "file");
    }
}
