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
    learner_profile = str(record.get("learner_profile", "working_memory_low")).strip() or "working_memory_low"
    difficulty_focus = str(record.get("difficulty_focus", "")).strip()
    problem_types = record.get("problem_types", [])

    lines = [
        "You are an English-learning accessibility assistant.",
        f"Task: {task}",
        f"Language: {language}",
        f"Target context: {target_context}",
        f"Learner profile: {learner_profile}",
    ]
    if difficulty_focus:
        lines.append(f"Difficulty focus: {difficulty_focus}")
    if isinstance(problem_types, list) and problem_types:
        lines.append(f"Problem types: {', '.join(str(item) for item in problem_types)}")
    lines.extend(
        [
            "Return strict JSON only.",
            f"Text: {text}",
        ]
    )

    return "\n".join(lines)


def normalize_record(record: dict[str, object]) -> dict[str, str]:
    task = str(record.get("task", "")).strip()
    if task not in SUPPORTED_TASKS:
        raise ValueError(f"unsupported task: {task}")
    if not isinstance(record.get("output"), dict):
        raise ValueError("output must be a JSON object")
    text = str(record.get("text", "")).strip()
    if not text:
        raise ValueError("text is required")
    _validate_output_shape(task, record["output"])

    return {
        "prompt": build_prompt(record),
        "target": json.dumps(record["output"], ensure_ascii=False, sort_keys=True),
        "task": task,
    }


def _validate_output_shape(task: str, output: object) -> None:
    if not isinstance(output, dict):
        raise ValueError("output must be a JSON object")

    if task == "chunking":
        segments = output.get("segments")
        if not isinstance(segments, list) or not segments:
            raise ValueError("chunking output.segments must be a non-empty list")
        return

    if task == "summary":
        summary = output.get("summary")
        if not isinstance(summary, str) or not summary.strip():
            raise ValueError("summary output.summary must be a non-empty string")
        return

    if task == "speaking_plan":
        opener_options = output.get("opener_options")
        steps = output.get("steps")
        if not isinstance(opener_options, list) or not opener_options:
            raise ValueError("speaking_plan output.opener_options must be a non-empty list")
        if not isinstance(steps, list) or not steps:
            raise ValueError("speaking_plan output.steps must be a non-empty list")
        return


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", required=True)
    parser.add_argument("--output", required=True)
    args = parser.parse_args()

    input_path = Path(args.input)
    output_path = Path(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)

    with input_path.open("r", encoding="utf-8-sig") as source, output_path.open("w", encoding="utf-8") as target:
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
