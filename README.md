# Secure Coding Support RAG

A local Rust agent built with [Rig](https://github.com/0xPlaygrounds/rig) and [Ollama](https://ollama.com/).
Ask a secure coding question, or point it at a source file. It explains the risks and shows a safer version.
Everything runs locally. It needs no API keys.

## Setup

```
ollama pull nomic-embed-text
ollama pull CyberCrew/notmythos-8b:latest
```

Put your secure coding PDFs in `docs/`. If `docs/` has no readable PDF, the app uses a small built-in set of rules.

## Run

```
cargo run
```

| Command | Action |
| :--- | :--- |
| any text | Ask a secure coding question |
| `review <path>` | Review a source file, 60 lines at a time |
| `save` | Save the last review to `output/review-<file>.md` |
| `exit` | Quit |

Set `EMBED_MODEL` or `COMPLETION_MODEL` to use other Ollama models.

## Answer format

Each answer has four sections: *Findings*, *Secure fix*, *Why it works*, *How to verify*.
Lines that come only from the model are marked `(model knowledge)`. Unsure items say `needs review`.

## Test

```
cargo test
```

`tests/samples/flawed/` has 10 files with known flaws. `tests/samples/safe/` has 3 safe files.
Run `review` on each file to check the agent.

## Limits

An 8B-class model is not a static analyser. It can miss flaws, flag safe code, and suggest fixes that need changes.
Test every fix. Keep using a SAST tool and human code review.
