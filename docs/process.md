# Question Desk Process Guide

This document explains how Question Desk was built, what each part means, and how the pieces work together.

## 1. Starting point

The project began as a Tauri starter app. Tauri combines:

- A web frontend for the interface.
- A Rust backend for native desktop behavior.
- A Vite development server for fast frontend builds.
- Cargo for Rust dependencies and compilation.

The original starter only opened an empty window. The application logic was added in stages.

## 2. Repository setup

The project lives in the `question-desk` folder. That folder has its own Git repository and GitHub remote:

```text
https://github.com/eugene668/question-desk
```

The important Git steps were:

1. Initialize the project repository on `main`.
2. Connect it to the existing GitHub repository.
3. Merge the existing remote placeholder README with the local scaffold.
4. Commit the completed workflow.
5. Push `main` to GitHub.

The final commits are:

- `229f0ee` - initial Tauri scaffold.
- `80d4657` - merge the existing remote README with the scaffold.
- `46c8cd9` - complete the Question Desk workflow.
- `8b37e4c` - ignore Python bytecode generated during validation.

A clean Git status means there are no local changes waiting to be committed.

## 3. M1: frontend and desktop scaffold

The main project files are:

- `index.html` - the HTML structure loaded by the webview.
- `src/main.ts` - frontend behavior and event handling.
- `src/styles.css` - the visual design.
- `src-tauri/src/main.rs` - the native executable entry point.
- `src-tauri/src/lib.rs` - the Tauri commands and application logic.
- `src-tauri/tauri.conf.json` - Tauri window, build, and bundle configuration.

`npm install` downloads the JavaScript dependencies declared in `package.json`. In particular, it installs the local Tauri CLI used by `npm run tauri dev`.

`npm run tauri dev` starts two processes:

1. Vite serves the frontend at `http://localhost:1420`.
2. Cargo compiles and launches the Rust desktop application, which opens the Tauri window.

The command must be run from the directory containing `package.json`, which is `question-desk`, not its parent folder.

## 4. M2: local Rust store

The Rust backend stores all questions in one file named `questions.json` inside the operating system's application data directory. It creates the directory when needed.

The data shape is:

```json
{
  "id": "q_20260920_120000123",
  "asked_at": "2026-09-20T12:00:00Z",
  "asker": "youth group",
  "context": "after a session",
  "question": "An invented training question",
  "tags": ["example", "training"],
  "draft": null
}
```

The Rust commands are:

- `save_question` - validates and writes a new question.
- `list_questions` - reads the JSON file and returns newest questions first.
- `delete_question` - removes a question by ID.
- `draft_answer` - creates and saves an AI draft.

A Tauri command is a Rust function that can be called safely by the frontend through `invoke`. Each command returns `Result`, which means it can return either a successful value or a readable error message.

## 5. M3: frontend workflow

The frontend form collects:

- The required question.
- Who asked it.
- The context or location.
- Comma-separated tags.

On save, `src/main.ts` calls:

```ts
invoke("save_question", { asker, context, question, tags });
```

After a successful save, the form clears and the list reloads from Rust. This is important because the frontend does not keep a second source of truth for the saved questions.

Each saved question shows its date, asker, context, tags, and controls to draft or delete. Delete requires confirmation. Command failures appear in the visible alert area instead of being silently ignored.

## 6. M4: AI drafting

The AI request is deliberately made by Rust, not by browser-side TypeScript. This keeps `ANTHROPIC_API_KEY` out of the webview and frontend bundle.

The local `.env` file is loaded by `dotenvy` when the Rust program starts. The real `.env` file is ignored by Git. `.env.example` documents the required variable without containing a secret.

The request goes to Anthropic's Messages API with:

- The `claude-sonnet-5` model alias.
- A system prompt requiring a humble, short starting response.
- A user message containing the saved question context.
- A JSON-only response contract.

The expected model output is:

```json
{
  "draft": "A short starting response.",
  "verify": [
    "Check the relevant source.",
    "Confirm the historical or factual claim."
  ]
}
```

The response is parsed before it is saved. If the key is missing, the network fails, Anthropic returns an error, or the response is not valid JSON, the command returns an error and the rest of the app keeps working.

An AI draft is not a final answer. The UI labels it `AI draft - verify before use`.

## 7. M5: weekly report

`scripts/report.py` reads the same `questions.json` file used by the app. It uses only Python's standard library, so no Python package installation is required.

Basic command:

```text
python scripts/report.py --days 7
```

The script:

1. Finds the platform-specific data file, unless `--file` is supplied.
2. Loads the JSON array.
3. Filters questions to the requested number of recent days.
4. Counts drafted questions.
5. Counts the most common tags.
6. Writes a CSV that can be shared with a team lead.

The CSV is an export, not a replacement for the JSON store. The app continues to use JSON for persistence.

## 8. M6: documentation and validation

The handoff files are:

- `README.md` - setup, API key configuration, report usage, data paths, and limitations.
- `docs/decisions.md` - short explanations of the main architecture choices.
- `docs/process.md` - this step-by-step process and terminology guide.

The validation commands used were:

```text
npm run build
cargo check
cargo fmt -- --check
python -m py_compile scripts/report.py
python scripts/report.py --file sample.json --days 7
```

The frontend build checks TypeScript and creates the Vite production bundle. `cargo check` compiles the Rust code without producing a release binary. `cargo fmt -- --check` checks Rust formatting. Python compilation catches syntax errors, while the report fixture check verifies the summary and CSV behavior.

## 9. Important terms

### Tauri
A desktop application toolkit that uses web technology for the interface and Rust for native capabilities.

### Vite
The frontend development server and production bundler. It serves the UI quickly during development and creates the `dist` output for builds.

### Cargo
Rust's package manager and build tool. It reads `Cargo.toml`, downloads Rust crates, and compiles the Tauri backend.

### Crate
A Rust package or library. For example, `chrono` handles timestamps, `serde` handles serialization, `reqwest` makes HTTP requests, and `dotenvy` loads `.env` files.

### Serialization
Converting a Rust value into JSON so it can be stored or sent over a boundary. Deserialization is converting JSON back into a Rust value.

### Tauri command
A Rust function exposed to the frontend through Tauri's `invoke` mechanism.

### API key
A secret credential that authorizes requests to an external service. It belongs in a local ignored environment file, never in source code, Git history, screenshots, or frontend JavaScript.

### CSV
Comma-separated values. It is a simple table format supported by spreadsheets and useful for sharing reports.

### Data path
The operating system location where the app stores `questions.json`. The path differs by Windows, macOS, and Linux, which is why the README documents all three.

## 10. Normal development loop

For future changes:

1. Open a terminal in `question-desk`.
2. Make one focused change.
3. Run the narrowest relevant check.
4. Run `npm run build` or `cargo check` when the change crosses frontend or Rust boundaries.
5. Review `git diff` and `git status`.
6. Commit with a clear conventional message such as `feat: add question search`.
7. Push the branch or open a pull request according to the repository workflow.
