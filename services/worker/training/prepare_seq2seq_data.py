from __future__ import annotations

import argparse
import json
from pathlib import Path


SUPPORTED_TASKS = {
    "chunking",
    "summary",
    "reader_plan",
    "listening_plan",
    "speaking_plan",
    "rescue_plan",
}


def build_prompt(record: dict[str, object]) -> str:
    task = str(record["task"]).strip()
    text = str(record["text"]).strip()
    language = str(record.get("language", "en")).strip() or "en"
    target_context = str(record.get("target_context", "general")).strip() or "general"

    return (
        "You are an English-learning accessibility assistant.\n"
        f"Task: {task}\n"
        f"Language: {language}\n"
        f"Target context: {target_context}\n"
        "Return strict JSON only.\n"
        f"Text: {text}"
    )


def normalize_record(record: dict[str, object]) -> dict[str, str]:
    task = str(record.get("task", "")).strip()
    if task not in SUPPORTED_TASKS:
        raise ValueError(f"unsupported task: {task}")
    if not isinstance(record.get("output"), dict):
        raise ValueError("output must be a JSON object")
    text = str(record.get("text", "")).strip()
    if not text:
        raise ValueError("text is required")

    return {
        "prompt": build_prompt(record),
        "target": json.dumps(record["output"], ensure_ascii=False, sort_keys=True),
        "task": task,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", required=True)
    parser.add_argument("--output", required=True)
    args = parser.parse_args()

    input_path = Path(args.input)
    output_path = Path(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)

    with input_path.open("r", encoding="utf-8") as source, output_path.open("w", encoding="utf-8") as target:
        for line_number, line in enumerate(source, start=1):
            raw = line.strip()
            if not raw:
                continue
            record = json.loads(raw)
            try:
                normalized = normalize_record(record)
            except ValueError as exc:
                raise ValueError(f"line {line_number}: {exc}") from exc
            target.write(json.dumps(normalized, ensure_ascii=False) + "\n")


if __name__ == "__main__":
    main()
