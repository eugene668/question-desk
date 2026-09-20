#!/usr/bin/env python3
"""Print a weekly Question Desk summary and write a shareable CSV."""

import argparse
import csv
import json
import os
import sys
from collections import Counter
from datetime import datetime, timedelta, timezone
from pathlib import Path

APP_IDENTIFIER = "africa.apakan.question-desk"


def default_store_path() -> Path:
    if sys.platform == "win32":
        root = Path(os.environ.get("APPDATA", Path.home() / "AppData" / "Roaming"))
        return root / APP_IDENTIFIER / "questions.json"
    if sys.platform == "darwin":
        return Path.home() / "Library" / "Application Support" / APP_IDENTIFIER / "questions.json"
    root = Path(os.environ.get("XDG_DATA_HOME", Path.home() / ".local" / "share"))
    return root / APP_IDENTIFIER / "questions.json"


def parse_timestamp(value: str) -> datetime:
    return datetime.fromisoformat(value.replace("Z", "+00:00")).astimezone(timezone.utc)


def load_questions(path: Path) -> list[dict]:
    if not path.exists():
        raise FileNotFoundError(
            f"Questions file not found at {path}. Run the app first or pass --file PATH."
        )
    try:
        with path.open(encoding="utf-8") as handle:
            data = json.load(handle)
    except (OSError, json.JSONDecodeError) as error:
        raise RuntimeError(f"Could not read {path}: {error}") from error
    if not isinstance(data, list):
        raise RuntimeError(f"Expected {path} to contain a JSON array.")
    return [item for item in data if isinstance(item, dict)]


def write_csv(questions: list[dict], output_path: Path) -> None:
    with output_path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(
            handle,
            fieldnames=["id", "asked_at", "asker", "question", "tags", "has_draft"],
        )
        writer.writeheader()
        for question in questions:
            writer.writerow(
                {
                    "id": question.get("id", ""),
                    "asked_at": question.get("asked_at", ""),
                    "asker": question.get("asker", ""),
                    "question": question.get("question", ""),
                    "tags": ", ".join(question.get("tags", [])),
                    "has_draft": bool(question.get("draft")),
                }
            )


def main() -> int:
    parser = argparse.ArgumentParser(description="Summarize recent Question Desk entries.")
    parser.add_argument("--days", type=int, default=7, help="How many days to include (default: 7).")
    parser.add_argument("--file", type=Path, help="Path to questions.json; overrides the platform default.")
    parser.add_argument("--output", type=Path, help="CSV output path (default: report_YYYY-MM-DD.csv).")
    args = parser.parse_args()
    if args.days < 1:
        parser.error("--days must be at least 1")

    source = args.file or default_store_path()
    try:
        questions = load_questions(source)
    except (FileNotFoundError, RuntimeError) as error:
        print(f"Error: {error}", file=sys.stderr)
        return 1

    cutoff = datetime.now(timezone.utc) - timedelta(days=args.days)
    recent = []
    for question in questions:
        try:
            if parse_timestamp(str(question.get("asked_at", ""))) >= cutoff:
                recent.append(question)
        except ValueError:
            continue

    tag_counts = Counter(
        tag.strip().lower()
        for question in recent
        for tag in question.get("tags", [])
        if isinstance(tag, str) and tag.strip()
    )
    drafted = sum(1 for question in recent if question.get("draft"))
    output_path = args.output or Path(f"report_{datetime.now().date().isoformat()}.csv")
    write_csv(recent, output_path)

    print(f"Question Desk - last {args.days} days")
    print(f"{len(recent)} questions logged, {drafted} drafted")
    if tag_counts:
        top_tags = ", ".join(f"{tag} ({count})" for tag, count in tag_counts.most_common(5))
        print(f"top tags: {top_tags}")
    else:
        print("top tags: none")
    print(f"wrote {output_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
