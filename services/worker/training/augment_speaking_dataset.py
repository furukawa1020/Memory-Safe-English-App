from __future__ import annotations

import argparse
import json
import re
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any


_JSON_BLOCK = re.compile(r"\{.*\}", re.DOTALL)


@dataclass(slots=True)
class Seq2SeqGenerator:
    model_name: str
    max_input_tokens: int = 512
    max_new_tokens: int = 256
    num_beams: int = 4
    temperature: float = 0.2
    cache_dir: str = ""
    _torch: Any = field(init=False, repr=False)
    _tokenizer: Any = field(init=False, repr=False)
    _model: Any = field(init=False, repr=False)

    def __post_init__(self) -> None:
        import torch
        from transformers import AutoModelForSeq2SeqLM, AutoTokenizer

        model_kwargs: dict[str, Any] = {}
        if self.cache_dir:
            model_kwargs["cache_dir"] = self.cache_dir

        self._torch = torch
        self._tokenizer = AutoTokenizer.from_pretrained(self.model_name, **model_kwargs)
        self._model = AutoModelForSeq2SeqLM.from_pretrained(self.model_name, **model_kwargs)
        self._model.eval()

    def generate_text(self, prompt: str) -> str:
        inputs = self._tokenizer(
            prompt,
            return_tensors="pt",
            truncation=True,
            max_length=self.max_input_tokens,
        )
        generation_kwargs = {
            "max_new_tokens": self.max_new_tokens,
            "num_beams": self.num_beams,
            "do_sample": self.temperature > 0,
            "temperature": self.temperature if self.temperature > 0 else None,
        }
        generation_kwargs = {key: value for key, value in generation_kwargs.items() if value is not None}

        with self._torch.no_grad():
            output_ids = self._model.generate(**inputs, **generation_kwargs)
        return self._tokenizer.decode(output_ids[0], skip_special_tokens=True)

    def generate_json(self, prompt: str) -> dict[str, Any]:
        return parse_json_payload(self.generate_text(prompt))


def parse_json_payload(text: str) -> dict[str, Any]:
    match = _JSON_BLOCK.search(text)
    candidate = match.group(0) if match else text
    try:
        payload = json.loads(candidate)
    except json.JSONDecodeError:
        return {}
    return payload if isinstance(payload, dict) else {}


def validate_speaking_output(output: dict[str, Any]) -> bool:
    opener_options = output.get("opener_options")
    bridge_phrases = output.get("bridge_phrases")
    steps = output.get("steps")
    rescue_phrases = output.get("rescue_phrases")
    summary = output.get("summary")
    return (
        isinstance(summary, str)
        and bool(summary.strip())
        and isinstance(opener_options, list)
        and len(opener_options) >= 1
        and isinstance(bridge_phrases, list)
        and len(bridge_phrases) >= 1
        and isinstance(steps, list)
        and len(steps) >= 1
        and isinstance(rescue_phrases, list)
        and len(rescue_phrases) >= 1
    )


def build_prompt(record: dict[str, Any], variant_index: int) -> str:
    output = record["output"]
    return (
        "You are creating accessible English speaking plans for learners with low working memory.\n"
        "Rewrite the speaking plan so it stays natural, short, and easy to hold in memory.\n"
        "Keep the same intent and target context.\n"
        f"Target context: {record.get('target_context', 'general')}\n"
        f"Learner profile: {record.get('learner_profile', 'working_memory_low')}\n"
        f"Difficulty focus: {record.get('difficulty_focus', 'sentence_holding')}\n"
        f"Problem types: {', '.join(record.get('problem_types', []))}\n"
        f"Variant index: {variant_index}\n"
        "Requirements:\n"
        "- opener_options should be natural spoken English\n"
        "- steps should stay short and complete\n"
        "- use simple bridge phrases\n"
        "- rescue phrases should be immediately usable\n"
        '- return strict JSON only with keys: "summary", "opener_options", "bridge_phrases", "steps", "rescue_phrases"\n'
        f"Input text: {record['text']}\n"
        f"Current output JSON: {json.dumps(output, ensure_ascii=False)}"
    )


def build_summary_prompt(record: dict[str, Any], variant_index: int) -> str:
    return (
        "Rewrite the summary for a learner with low working memory.\n"
        "Keep it to one short natural sentence.\n"
        f"Target context: {record.get('target_context', 'general')}\n"
        f"Variant index: {variant_index}\n"
        f"Input text: {record['text']}\n"
        f"Current summary: {record['output'].get('summary', '')}"
    )


def build_openers_prompt(record: dict[str, Any], variant_index: int) -> str:
    return (
        "Write two short natural opener sentences for a learner with low working memory.\n"
        "Return exactly two lines, one opener per line.\n"
        f"Target context: {record.get('target_context', 'general')}\n"
        f"Variant index: {variant_index}\n"
        f"Input text: {record['text']}\n"
        f"Current openers: {json.dumps(record['output'].get('opener_options', []), ensure_ascii=False)}"
    )


def build_steps_prompt(record: dict[str, Any], variant_index: int) -> str:
    steps = record["output"].get("steps", [])
    return (
        "Rewrite the speaking steps for a learner with low working memory.\n"
        "Keep the number of steps the same and make each step short and speakable.\n"
        "Return one line per step.\n"
        f"Target context: {record.get('target_context', 'general')}\n"
        f"Variant index: {variant_index}\n"
        f"Input text: {record['text']}\n"
        f"Current steps: {json.dumps(steps, ensure_ascii=False)}"
    )


def augment_record(generator: Seq2SeqGenerator, record: dict[str, Any], variant_index: int) -> dict[str, Any] | None:
    output = record["output"]
    summary_text = _normalize_single_line(
        generator.generate_text(build_summary_prompt(record, variant_index)),
        fallback=str(output.get("summary", "")),
    )
    opener_options = _normalize_openers(
        generator.generate_text(build_openers_prompt(record, variant_index)),
        fallback=[str(item) for item in output.get("opener_options", [])],
        limit=2,
    )
    step_lines = _normalize_step_lines(
        generator.generate_text(build_steps_prompt(record, variant_index)),
        fallback=[str(step.get("text", "")) for step in output.get("steps", []) if isinstance(step, dict)],
        limit=len(output.get("steps", [])) or 2,
    )
    base_steps = output.get("steps", [])
    rewritten_steps = []
    for index, base_step in enumerate(base_steps):
        if not isinstance(base_step, dict):
            continue
        rewritten_steps.append(
            {
                "step": base_step.get("step", index + 1),
                "text": step_lines[index] if index < len(step_lines) else str(base_step.get("text", "")),
                "purpose": base_step.get("purpose", "support"),
            }
        )

    payload = {
        "summary": summary_text,
        "opener_options": opener_options,
        "bridge_phrases": output.get("bridge_phrases", []),
        "steps": rewritten_steps,
        "rescue_phrases": output.get("rescue_phrases", []),
    }
    if not validate_speaking_output(payload):
        return None

    return {
        **record,
        "output": payload,
        "meta_source": record.get("meta_id", "seed"),
        "meta_variant": variant_index,
    }


def _normalize_single_line(text: str, *, fallback: str) -> str:
    lines = _extract_lines(text)
    candidate = lines[0] if lines else fallback
    return candidate.strip().strip("- ").strip()


def _normalize_lines(text: str, *, fallback: list[str], limit: int) -> list[str]:
    lines = _extract_lines(text)
    cleaned = [line.strip().strip("- ").strip() for line in lines if line.strip()]
    if not cleaned:
        cleaned = fallback
    return cleaned[:limit]


def _normalize_openers(text: str, *, fallback: list[str], limit: int) -> list[str]:
    parsed = _try_parse_list_literal(text)
    if parsed:
        cleaned = [_clean_sentence(item) for item in parsed if _looks_like_clean_sentence(item)]
        if cleaned:
            return cleaned[:limit]

    lines = [_clean_sentence(line) for line in _extract_lines(text)]
    lines = [line for line in lines if _looks_like_clean_sentence(line)]
    if not lines:
        lines = [_clean_sentence(line) for line in fallback if _looks_like_clean_sentence(line)]
    return lines[:limit]


def _normalize_step_lines(text: str, *, fallback: list[str], limit: int) -> list[str]:
    lines = [_clean_sentence(line) for line in _extract_lines(text)]
    valid = [line for line in lines if _looks_like_clean_step(line)]
    if not valid:
        valid = [_clean_sentence(line) for line in fallback if _looks_like_clean_step(line)]
    if len(valid) < limit:
        fallback_clean = [_clean_sentence(line) for line in fallback if _looks_like_clean_step(line)]
        for item in fallback_clean:
            if len(valid) >= limit:
                break
            if item not in valid:
                valid.append(item)
    return valid[:limit]


def _extract_lines(text: str) -> list[str]:
    raw_lines = [line.strip() for line in text.splitlines() if line.strip()]
    if raw_lines:
        return raw_lines
    return [part.strip() for part in text.split("||") if part.strip()]


def _try_parse_list_literal(text: str) -> list[str]:
    stripped = text.strip()
    if not stripped:
        return []
    try:
        payload = json.loads(stripped)
    except json.JSONDecodeError:
        return []
    if isinstance(payload, list):
        return [str(item) for item in payload if isinstance(item, str)]
    return []


def _clean_sentence(text: str) -> str:
    cleaned = text.strip().strip('"').strip("'").strip()
    cleaned = re.sub(r"\s+", " ", cleaned)
    cleaned = cleaned.removeprefix("Step1:").removeprefix("step 1:").strip()
    return cleaned


def _looks_like_clean_sentence(text: str) -> bool:
    if not text or len(text) > 120:
        return False
    bad_markers = ['{"', '["', '"]', 'step 1:', '"purpose"', '"step"']
    lowered = text.lower()
    return not any(marker in lowered for marker in bad_markers)


def _looks_like_clean_step(text: str) -> bool:
    if not _looks_like_clean_sentence(text):
        return False
    if len(text.split()) > 16:
        return False
    if text.count(":") > 1:
        return False
    return True


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", required=True)
    parser.add_argument("--output", required=True)
    parser.add_argument("--model-name", default="google/flan-t5-base")
    parser.add_argument("--variants-per-record", type=int, default=1)
    parser.add_argument("--limit", type=int, default=0)
    parser.add_argument("--max-input-tokens", type=int, default=512)
    parser.add_argument("--max-new-tokens", type=int, default=256)
    parser.add_argument("--num-beams", type=int, default=4)
    parser.add_argument("--temperature", type=float, default=0.2)
    parser.add_argument("--cache-dir", default="")
    args = parser.parse_args()

    input_path = Path(args.input)
    output_path = Path(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)

    generator = Seq2SeqGenerator(
        model_name=args.model_name,
        max_input_tokens=args.max_input_tokens,
        max_new_tokens=args.max_new_tokens,
        num_beams=args.num_beams,
        temperature=args.temperature,
        cache_dir=args.cache_dir,
    )

    total_written = 0
    with input_path.open("r", encoding="utf-8-sig") as source, output_path.open("w", encoding="utf-8") as target:
        for line_index, line in enumerate(source):
            if args.limit and line_index >= args.limit:
                break
            raw = line.strip()
            if not raw:
                continue
            record = json.loads(raw)
            if record.get("task") != "speaking_plan":
                continue
            for variant_index in range(1, args.variants_per_record + 1):
                augmented = augment_record(generator, record, variant_index)
                if augmented is None:
                    continue
                target.write(json.dumps(augmented, ensure_ascii=False) + "\n")
                total_written += 1

    print(f"wrote {total_written} augmented records")


if __name__ == "__main__":
    main()
