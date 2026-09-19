# Question Desk — starter scaffold

This is the **bare scaffold only** for Forward Deployed Expertise Training,
Project 01. It's a Tauri + Vanilla TypeScript app that opens an empty
"Question Desk" window and does nothing else yet. The `greet` demo command
has been removed on purpose — you're building the real commands.

See the full project brief (`AI_Skill_builder_Project_01_Question_Desk.pdf`)
for what to build and in what order (M0–M6).

## Using this scaffold

1. Create an empty repo on GitHub named `question-desk` (private is fine).
2. Unzip this folder's contents into your clone of that repo — don't zip the
   repo itself, copy the files *into* it — so `README.md`, `src/`,
   `src-tauri/`, etc. sit at the repo root next to `.git/`.
3. Commit it as your starting point:
   ```
   git add .
   git commit -m "chore: scaffold tauri app"
   git push
   ```
4. Install and run:
   ```
   npm install
   npm run tauri dev
   ```
   An empty "Question Desk" window should open. If it does, you're on M1 and
   ready to start M2 (the Rust store).

## What's here

- `index.html` / `src/main.ts` / `src/styles.css` — empty frontend shell.
- `src-tauri/src/lib.rs` — Tauri app entry point, no commands registered yet.
- `src-tauri/tauri.conf.json` — app renamed to Question Desk.
- `.gitignore` / `.env.example` — see below.

## Setting your API key (needed from M4 onward)

Copy `.env.example` to `.env` and fill in your key:

```
cp .env.example .env
```

`.env` is git-ignored — it should never be committed. Load it in Rust with
the `dotenvy` crate, per the project brief.

## Prerequisites

If `npm install` or `npm run tauri dev` fail, re-check the
[Toolchain Setup Guide](../AI_Skill_builder_Set_up.pdf) — this scaffold
assumes all 8 setup steps already pass.
