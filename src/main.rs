//! Secure coding support agent built on the Rig framework.
//! Uses a local Ollama server for embeddings (nomic-embed-text) and completion.
//! Usage: `cargo run`, then ask a question, or type `review <path>`, `save`, `exit`.

#![forbid(unsafe_code)]

mod review;

use anyhow::{Context, Result};
use review::{
    build_prompt, build_question_prompt, detect_language, review_slug, split_code, MAX_PIECE_LINES,
};
use rig::prelude::*;
use rig::{
    embeddings::EmbeddingsBuilder, loaders::PdfFileLoader,
    vector_store::in_memory_store::InMemoryVectorStore, Embed,
};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, Write};
use std::path::Path;

const CHUNK_SIZE: usize = 1500;
const TOP_N: usize = 3;
const MAX_FILE_BYTES: u64 = 1_000_000;

const PREAMBLE: &str = "You are a secure coding assistant for developers. \
Always answer in these Markdown sections, in this order: \
### Findings (the issue, the affected line, the risk, and the CWE or OWASP category), \
### Secure fix (corrected code), \
### Why it works, \
### How to verify (a test or a check). \
Use the retrieved context first. Mark each line that comes only from your own knowledge with `(model knowledge)`. \
If the code looks safe, say so and do not invent issues. \
Never produce working exploit payloads. \
These patterns are safe, do not flag them: parameterized queries, os.urandom, the secrets module, scrypt, Argon2, bcrypt, canonical path checks inside a base directory, textContent, and verify=True. \
Only report a problem if you can quote the exact unsafe line, and that line must really be unsafe. A line that already uses a safe pattern is not a finding. \
If the code has no problem, write only `No issues found` under ### Findings and skip the other sections. \
Never invent CVE IDs, versions or commands. \
Write `needs review` when you are unsure.";

#[derive(Embed, Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
struct Document {
    id: String,
    #[embed]
    content: String,
}

/// Splits text content into chunks of at most `chunk_size` characters along whitespace boundaries.
pub fn chunk_text(content: &str, chunk_size: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();
    for word in content.split_whitespace() {
        if current.len() + word.len() + 1 > chunk_size && !current.is_empty() {
            chunks.push(std::mem::take(&mut current).trim().to_string());
        }
        current.push_str(word);
        current.push(' ');
    }
    if !current.is_empty() {
        chunks.push(current.trim().to_string());
    }
    chunks
}

fn load_pdf(path: &Path) -> Result<Vec<String>> {
    let content_chunks = PdfFileLoader::with_glob(path.to_str().context("Invalid path")?)?
        .read()
        .into_iter()
        .filter_map(|result| {
            result
                .map_err(|e| {
                    eprintln!("Error reading PDF content: {e}");
                    e
                })
                .ok()
        })
        .flat_map(|content| chunk_text(&content, CHUNK_SIZE))
        .collect::<Vec<_>>();
    if content_chunks.is_empty() {
        anyhow::bail!("No content found in PDF file: {}", path.display());
    }
    Ok(content_chunks)
}

/// Built-in secure coding rules, used when `docs/` has no readable PDFs.
fn builtin_corpus() -> Vec<String> {
    [
        "SQL injection (CWE-89): never build SQL by string concatenation or formatting. Use parameterized queries or prepared statements and bind user input as parameters. Use an allow-list for table and column names.",
        "Cross-site scripting (CWE-79): encode output for the context where it is used (HTML body, attribute, JavaScript, URL). Prefer auto-escaping templates. Do not insert untrusted data with innerHTML.",
        "Password storage (CWE-256, CWE-916): never store passwords in plain text or with fast hashes such as MD5 or SHA-1. Use a slow salted password hash such as Argon2id, scrypt or bcrypt.",
        "Input validation (CWE-20): validate type, length, range and format of all untrusted input on the server. Prefer allow-lists over deny-lists. Reject invalid input early.",
        "Hard-coded secrets (CWE-798): do not put passwords, API keys or tokens in source code. Load them from environment variables or a secrets manager. Rotate any secret that was committed.",
        "Path traversal (CWE-22): do not join untrusted input into file paths. Resolve the canonical path and check that it stays inside the allowed base directory. Reject '..' and absolute paths.",
        "Unsafe deserialization (CWE-502): do not deserialize untrusted data with pickle, Java ObjectInputStream or similar. Use a data-only format such as JSON with schema validation.",
        "OS command injection (CWE-78): do not pass untrusted input to a shell. Call programs with an argument list and no shell. Validate arguments against an allow-list.",
        "Broken access control (CWE-862, CWE-639): check authorization on the server for every request and every object. Deny by default. Do not trust client-supplied IDs without an ownership check.",
        "Cryptography (CWE-327, CWE-330): use well-reviewed libraries and modern algorithms (AES-GCM, ChaCha20-Poly1305, SHA-256 or better). Use a cryptographically secure random generator for tokens and keys. Do not write your own crypto.",
        "Error handling and logging (CWE-209, CWE-532): do not show stack traces or internal details to users. Do not log passwords, tokens or personal data.",
        "Transport security (CWE-319, CWE-295): use TLS for all network traffic. Never disable certificate verification.",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

/// Loads PDF chunks from `docs/`, or falls back to the built-in corpus.
fn load_knowledge(docs_dir: &Path) -> Vec<String> {
    let mut chunks = Vec::new();
    if docs_dir.exists() {
        match load_pdf(&docs_dir.join("*.pdf")) {
            Ok(c) => chunks.extend(c),
            Err(e) => eprintln!("No PDF knowledge loaded ({e}). Using built-in rules."),
        }
    }
    if chunks.is_empty() {
        chunks = builtin_corpus();
    }
    chunks
}

/// Reads a source file with a size limit.
fn read_source(path: &str) -> Result<String> {
    let meta = std::fs::metadata(path).with_context(|| format!("cannot open {path}"))?;
    if !meta.is_file() {
        anyhow::bail!("{path} is not a file");
    }
    if meta.len() > MAX_FILE_BYTES {
        anyhow::bail!("{path} is larger than {MAX_FILE_BYTES} bytes");
    }
    std::fs::read_to_string(path).with_context(|| format!("cannot read {path} as UTF-8 text"))
}

/// Saves a review as Markdown and returns the path.
fn save_review(file: &str, markdown: &str) -> Result<String> {
    std::fs::create_dir_all("output")?;
    let path = format!("output/review-{}.md", review_slug(file));
    std::fs::write(&path, markdown).with_context(|| format!("writing {path}"))?;
    Ok(path)
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = rig::providers::ollama::OllamaConfig::new()
        .with_base_url("http://localhost:11434")
        .client();

    let embed_model_name =
        std::env::var("EMBED_MODEL").unwrap_or_else(|_| "nomic-embed-text".to_string());
    let completion_model_name = std::env::var("COMPLETION_MODEL")
        .unwrap_or_else(|_| "CyberCrew/notmythos-8b:latest".to_string());

    let embedding_model = client.embedding(&embed_model_name, None).erase();

    let chunks = load_knowledge(&std::env::current_dir()?.join("docs"));
    println!("Loaded {} security chunks.", chunks.len());

    let mut builder = EmbeddingsBuilder::new(embedding_model.clone());
    for (i, chunk) in chunks.into_iter().enumerate() {
        builder = builder.document(Document {
            id: format!("secure_coding_{i}"),
            content: chunk,
        })?;
    }
    let embeddings = builder.build().await?;
    println!("Successfully generated embeddings");

    let vector_store = InMemoryVectorStore::from_documents(embeddings);
    let index = vector_store.index(embedding_model);

    // `native_completion` (/api/chat) is required: the OpenAI-compatible `completion` refuses `options`.
    let agent = AgentBuilder::new(client.native_completion(&completion_model_name))
        .preamble(PREAMBLE)
        .dynamic_context(TOP_N, index)
        .temperature(0.1)
        .additional_params(rig::serde_json::json!({"options": {"num_ctx": 8192}}))
        .build();

    println!(
        "Starting secure coding support chatbot (completion model: {completion_model_name})..."
    );
    println!("Ask a question, or type: review <path> | save | exit");

    let mut last_review: Option<(String, String)> = None;
    let stdin = std::io::stdin();
    loop {
        print!("> ");
        std::io::stdout().flush()?;
        let mut line = String::new();
        if stdin.lock().read_line(&mut line)? == 0 {
            break;
        }
        let input = line.trim();
        if input.is_empty() {
            continue;
        }
        match input {
            "exit" | "quit" => break,
            "save" => {
                match &last_review {
                    Some((file, md)) => match save_review(file, md) {
                        Ok(p) => println!("Saved {p}"),
                        Err(e) => eprintln!("Error: {e:#}"),
                    },
                    None => println!("Nothing to save. Run `review <path>` first."),
                }
                continue;
            }
            _ => {}
        }

        if let Some(path) = input.strip_prefix("review ").map(str::trim) {
            let code = match read_source(path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error: {e:#}");
                    continue;
                }
            };
            let language = detect_language(path).unwrap_or("unknown language");
            let pieces = split_code(&code, MAX_PIECE_LINES);
            if pieces.is_empty() {
                println!("File has no code to review.");
                continue;
            }
            let total = pieces.len();
            let mut md = format!("# Secure code review: {path}\n\n- **Language:** {language}\n- **Model:** {completion_model_name}\n\n> Draft from a local model. Test every fix. Keep using SAST tools and human review.\n");
            for (i, piece) in pieces.iter().enumerate() {
                println!("[{}/{total}] lines {}-{}", i + 1, piece.start, piece.end);
                match agent.prompt(build_prompt(language, path, piece)).await {
                    Ok(answer) => {
                        println!("{answer}\n");
                        md.push_str(&format!(
                            "\n## Lines {}-{}\n\n{}\n",
                            piece.start,
                            piece.end,
                            answer.to_string().trim()
                        ));
                    }
                    Err(e) => eprintln!("Error on lines {}-{}: {e}", piece.start, piece.end),
                }
            }
            last_review = Some((path.to_string(), md));
        } else {
            match agent.prompt(build_question_prompt(input)).await {
                Ok(answer) => println!("{answer}\n"),
                Err(e) => eprintln!("Error: {e}"),
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_text_empty_string() {
        assert!(chunk_text("", 100).is_empty());
    }

    #[test]
    fn test_chunk_text_whitespace_only() {
        assert!(chunk_text("   \n\t   \r\n  ", 100).is_empty());
    }

    #[test]
    fn test_chunk_text_short_content() {
        let text = "Use parameterized queries.";
        let chunks = chunk_text(text, 100);
        assert_eq!(chunks, vec![text.to_string()]);
    }

    #[test]
    fn test_chunk_text_splits_at_word_boundary() {
        let text = vec!["vulnerability"; 20].join(" ");
        let chunks = chunk_text(&text, 50);
        assert!(chunks.len() > 1);
        for chunk in &chunks {
            assert!(chunk.len() <= 50);
            for word in chunk.split_whitespace() {
                assert_eq!(word, "vulnerability");
            }
        }
    }

    #[test]
    fn test_chunk_text_single_word_longer_than_chunk_size() {
        let long_token = "A".repeat(120);
        let chunks = chunk_text(&long_token, 50);
        assert_eq!(chunks, vec![long_token]);
    }

    #[test]
    fn test_chunk_text_retains_all_words() {
        let text = "SQL injection occurs when untrusted data is sent to an interpreter as part of a command or query.";
        let chunks = chunk_text(text, 30);
        let rebuilt: Vec<&str> = chunks.iter().flat_map(|c| c.split_whitespace()).collect();
        assert_eq!(text.split_whitespace().collect::<Vec<_>>(), rebuilt);
    }

    #[test]
    fn test_builtin_corpus_fits_chunk_size() {
        let c = builtin_corpus();
        assert!(c.len() >= 5);
        assert!(c.iter().all(|s| s.len() <= CHUNK_SIZE));
    }

    #[test]
    fn test_load_knowledge_falls_back_when_docs_missing() {
        let chunks = load_knowledge(Path::new("definitely-missing-docs-dir"));
        assert_eq!(chunks, builtin_corpus());
    }

    #[test]
    fn test_read_source_rejects_missing_file() {
        assert!(read_source("definitely-missing-file.py").is_err());
    }
}
