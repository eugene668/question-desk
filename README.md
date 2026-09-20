# Question Desk

Question Desk is a small Tauri desktop app for logging difficult questions locally and preparing a careful AI-assisted first response. Questions and drafts stay in one JSON file owned by the Rust side of the app.

## Run it

From a fresh clone:

```text
npm install
npm run tauri dev
```

The app requires Node.js, Rust, and the Tauri Windows WebView2 prerequisite from the setup guide. The project root is the directory containing this README and `package.json`.

To build the frontend only:

```text
npm run build
```

## API key

AI drafting is optional. Without a key, saving, listing, and deleting questions continue to work and the Draft action shows a clear error.

For real drafts, copy `.env.example` to `.env` and fill in the key:

```text
ANTHROPIC_API_KEY=your-key-here
```

`.env` is ignored and must never be committed. The key is read only by Rust; it is never sent to the frontend.

The draft request uses Anthropic's Messages API with the `claude-sonnet-5` model alias. The response must be JSON with a `draft` string and 2 to 4 `verify` items.

## Weekly report

The report uses Python's standard library only. With the app's default data location:

```text
python scripts/report.py --days 7
```

You can provide a file explicitly when testing or moving data:

```text
python scripts/report.py --file path/to/questions.json --days 7 --output team-report.csv
```

The default `questions.json` locations are:

- Windows: `%APPDATA%\africa.apakan.question-desk\questions.json`
- macOS: `~/Library/Application Support/africa.apakan.question-desk/questions.json`
- Linux: `~/.local/share/africa.apakan.question-desk/questions.json` (or `$XDG_DATA_HOME/africa.apakan.question-desk/questions.json`)

The script prints the count logged, count drafted, and most common tags, then writes `report_YYYY-MM-DD.csv` with `id`, `asked_at`, `asker`, `question`, `tags`, and `has_draft` columns.

## Known limitations

- AI drafts require network access and a valid Anthropic API key.
- The app uses a single JSON file, so it is intended for one local user rather than concurrent multi-device editing.
- The AI output is a starting point only. Verify claims and sources before using it.
