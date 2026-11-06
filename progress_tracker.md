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
- [ ] Install Ollama.
- [ ] Pull the model `nomic-embed-text`.
- [ ] Pull the model `CyberCrew/notmythos-8b:latest`.
- [ ] Run `ollama show` on the model. Record its context length.
- [ ] Confirm at run time that `agent.prompt()` works with `dynamic_context`.

## Phase 2: Ingestion and index

- [x] Write `chunk_text` (1,500 characters, word boundaries).
- [x] Load `docs/*.pdf` with `PdfFileLoader`.
- [x] Add the built-in secure coding rules as a fallback.
- [x] Use the fallback when `docs/` is missing or empty.
- [x] Embed each chunk with `EmbeddingsBuilder`.
- [x] Store the embeddings in `InMemoryVectorStore`.
- [x] Write unit tests for chunking.
- [x] Write unit tests for the fallback.
- [ ] Put the secure coding PDFs in `docs/`.
- [ ] Run the app with the PDFs. Check the chunk count.

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
- [x] Set temperature to 0.2.
- [x] Set `num_ctx` to 8192.
- [x] Use `native_completion` for the Ollama chat API.
- [ ] Check at run time that the answers follow the sections.

## Phase 5: CLI

- [x] Write a stdin loop.
- [x] Send a plain question to `agent.prompt()`.
- [x] Add the `review <path>` command.
- [x] Print the progress as `[n/total] lines a-b`.
- [x] Add the `save` command. It writes `output/review-<file>.md`.
- [x] Add the `exit` command.
- [x] Reject files that are missing, not files, or larger than 1 MB.
- [ ] Test the CLI with Ollama running.
- [ ] Decide if Rig chat history is useful for follow-up questions.

## Phase 6: Test and tune

- [ ] Write 10 code samples with known flaws:
  - [ ] SQL built by string concatenation.
  - [ ] Plain-text password storage.
  - [ ] Hard-coded secret.
  - [ ] Unvalidated file path.
  - [ ] Unsafe deserialization.
  - [ ] Five more flaw types.
- [ ] Write 3 safe code samples.
- [ ] Check that the agent finds the flaws.
- [ ] Check that the agent does not flag the safe code.
- [ ] Check that the fixes compile.
- [ ] Tune chunk size.
- [ ] Tune top-N.
- [ ] Tune piece size.
- [ ] Tune temperature.
- [ ] Write `README.md`.

## Build checks

- [x] `cargo test` passes (18 tests).
- [x] `cargo clippy` shows no warnings.
- [ ] Install `rustfmt`. Then make `cargo fmt --check` pass.
- [ ] Do the first full run with Ollama.
