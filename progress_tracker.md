# Progress Tracker: Secure Coding Support Agent

Plan: `SECURE_CODING_SUPPORT_RAG_PLAN.md`
Framework: Rig 0.44 (Rust) with Ollama.

Key: `[x]` = done. `[ ]` = not done.

## Phase 1: Project setup

- [x] Create `Cargo.toml` with `rig`, `anyhow`, `serde` and `tokio`.
- [x] Add `.gitignore` with `target/` and `output/`.
- [x] Create the `docs/` folder.
- [x] Create the `output/` folder.
- [x] Set `forbid(unsafe_code)` in `main.rs`.
- [x] Confirm that `AgentBuilder` accepts `.temperature()` and `.additional_params()`. (The project compiles.)
- [x] Confirm that `dynamic_context` compiles with the agent.
- [x] Install Ollama.
- [x] Pull the model `nomic-embed-text`.
- [x] Pull the model `CyberCrew/notmythos-8b:latest`.
- [x] Run `ollama show` on the model. Record its context length.
- [x] Confirm at run time that `agent.prompt()` works with `dynamic_context`.

## Phase 2: Ingestion and index

- [x] Write `chunk_text` (1,500 characters, word boundaries).
- [x] Load `docs/*.pdf` with `PdfFileLoader`.
- [x] Add the built-in secure coding rules as a fallback.
- [x] Use the fallback when `docs/` is missing or empty.
- [x] Embed each chunk with `EmbeddingsBuilder`.
- [x] Store the embeddings in `InMemoryVectorStore`.
- [x] Write unit tests for chunking.
- [x] Write unit tests for the fallback.
- [x] Put the secure coding PDFs in `docs/`.
- [x] Run the app with the PDFs. Check the chunk count.

## Phase 3: Review helpers (`src/review.rs`)

- [x] Detect the language from the file extension.
- [x] Split code into pieces of 60 lines or fewer.
- [x] Make sure no piece splits a line.
- [x] Add real line numbers to each piece.
- [x] Build the prompt for a code piece.
- [x] Build the prompt for a plain question.
- [x] Make a safe file name for saved reviews.
- [x] Write unit tests for all helpers.

## Phase 4: Secure coding agent

- [x] Write the secure coding preamble.
- [x] Set the answer sections: Findings, Secure fix, Why it works, How to verify.
- [x] Tell the model to mark its own knowledge as `(model knowledge)`.
- [x] Tell the model to write `needs review` when it is unsure.
- [x] Tell the model not to produce working exploit payloads.
- [x] Set `dynamic_context(3, index)`.
- [x] Set temperature to 0.1 (lowered from 0.2 after tuning).
- [x] Set `num_ctx` to 8192.
- [x] Use `native_completion` for the Ollama chat API.
- [x] Check at run time that the answers follow the sections.

## Phase 5: CLI

- [x] Write a stdin loop.
- [x] Send a plain question to `agent.prompt()`.
- [x] Add the `review <path>` command.
- [x] Print the progress as `[n/total] lines a-b`.
- [x] Add the `save` command. It writes `output/review-<file>.md`.
- [x] Add the `exit` command.
- [x] Reject files that are missing, not files, or larger than 1 MB.
- [x] Test the CLI with Ollama running.
- [x] Decide if Rig chat history is useful for follow-up questions.

## Phase 6: Test and tune

- [x] Write 10 code samples with known flaws:
  - [x] SQL built by string concatenation.
  - [x] Plain-text password storage.
  - [x] Hard-coded secret.
  - [x] Unvalidated file path.
  - [x] Unsafe deserialization.
  - [x] Five more flaw types.
- [x] Write 3 safe code samples.
- [x] Check that the agent finds the flaws.
- [ ] Check that the agent does not flag the safe code. (2 of 3 safe samples flagged. See open items.)
- [x] Check that the fixes compile.
- [ ] Tune chunk size. (Blocked: the index has 1 chunk. See open items.)
- [ ] Tune top-N. (Blocked: the index has 1 chunk. See open items.)
- [x] Tune piece size. (60 lines works for all samples.)
- [x] Tune temperature.
- [x] Write `README.md`.

## Test results (last run)

- Model: `CyberCrew/notmythos-8b:latest`. It is a 3.2B Llama model, not 8B. Context length: 131072.
- Flawed samples: the agent found the flaw in 10 of 10 files.
- Fixes: all 13 fixes are valid code (`py_compile` and `node --check`). No fix was run.
- Safe samples: the agent flagged 2 of 3 as risky (`safe_hash.py`, `safe_query.py`). The check "does not flag the safe code" is **not** met.
- Weak fix: for `path_traversal.py` the fix only blocks names that start with `..`. Test every fix.
- Settings: chunk 1,500 characters. Top-N 3. Piece 60 lines. Temperature 0.1.
- `docs/` holds one PDF, so the index has 1 chunk. Chunk size and top-N cannot change the result until `docs/` has more PDFs.
- Chat history: not added. Each question stands alone. This keeps the prompt small.

## Open items

- [ ] Check that the agent does not flag the safe code. (Try a larger model or a second review pass.)
- [ ] Add more PDFs to `docs/`. Tune chunk size and top-N again.

## Build checks

- [x] `cargo test` passes (18 tests).
- [x] `cargo clippy` shows no warnings.
- [x] Install `rustfmt`. Then make `cargo fmt --check` pass.
- [x] Do the first full run with Ollama.
