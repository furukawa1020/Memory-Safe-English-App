from __future__ import annotations

import csv
import json
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any, Iterable


OPEN_LICENSE_MARKERS = (
    "public domain",
    "cc by",
    "cc-by",
    "cc0",
    "mit",
    "cc by-sa",
    "cc-by-sa",
)
RESTRICTED_LICENSE_MARKERS = (
    "all rights reserved",
    "cc by-nc",
    "cc-by-nc",
    "cc by-nd",
    "cc-by-nd",
    "cc by-nc-sa",
    "cc-by-nc-sa",
    "cc by-nc-nd",
    "cc-by-nc-nd",
)


@dataclass(slots=True)
class CorpusRecord:
    record_id: str
    source: str
    source_type: str
    title: str
    text: str
    language: str = "en"
    target_context: str = "general"
    difficulty_band: str = "intermediate"
    license: str = ""
    metadata: dict[str, Any] | None = None

    def to_dict(self) -> dict[str, Any]:
        payload = asdict(self)
        payload["metadata"] = self.metadata or {}
        return payload


def write_corpus_jsonl(records: Iterable[CorpusRecord], output_path: Path) -> int:
    output_path.parent.mkdir(parents=True, exist_ok=True)
    count = 0
    with output_path.open("w", encoding="utf-8") as handle:
        for record in records:
            handle.write(json.dumps(record.to_dict(), ensure_ascii=False) + "\n")
            count += 1
    return count


def read_corpus_jsonl(path: Path) -> list[CorpusRecord]:
    records: list[CorpusRecord] = []
    with path.open("r", encoding="utf-8-sig") as handle:
        for line in handle:
            raw = line.strip()
            if not raw:
                continue
            payload = json.loads(raw)
            records.append(
                CorpusRecord(
                    record_id=str(payload["record_id"]),
                    source=str(payload["source"]),
                    source_type=str(payload["source_type"]),
                    title=str(payload.get("title", "")).strip(),
                    text=str(payload.get("text", "")).strip(),
                    language=str(payload.get("language", "en")).strip() or "en",
                    target_context=str(payload.get("target_context", "general")).strip() or "general",
                    difficulty_band=str(payload.get("difficulty_band", "intermediate")).strip() or "intermediate",
                    license=str(payload.get("license", "")).strip(),
                    metadata=dict(payload.get("metadata", {})),
                )
            )
    return records


def load_squad_records(
    input_path: Path,
    *,
    target_context: str = "general",
    limit: int | None = None,
) -> list[CorpusRecord]:
    with input_path.open("r", encoding="utf-8-sig") as handle:
        payload = json.load(handle)

    entries = payload.get("data", [])
    records: list[CorpusRecord] = []
    seen_texts: set[str] = set()
    for article_index, article in enumerate(entries):
        title = str(article.get("title", f"article-{article_index + 1}")).strip()
        for paragraph_index, paragraph in enumerate(article.get("paragraphs", [])):
            text = str(paragraph.get("context", "")).strip()
            if not text or text in seen_texts:
                continue
            qas = list(paragraph.get("qas", []))
            impossible_count = sum(1 for qa in qas if qa.get("is_impossible"))
            record = CorpusRecord(
                record_id=f"squad-{article_index + 1}-{paragraph_index + 1}",
                source="squad_v2",
                source_type="reading_qa",
                title=title,
                text=text,
                target_context=target_context,
                difficulty_band=_infer_text_difficulty(text),
                license="CC BY-SA 4.0",
                metadata={
                    "question_count": len(qas),
                    "impossible_count": impossible_count,
                    "article_title": title,
                },
            )
            records.append(record)
            seen_texts.add(text)
            if limit is not None and len(records) >= limit:
                return records
    return records


def load_clear_records(
    input_path: Path,
    *,
    target_context: str = "general",
    limit: int | None = None,
    open_license_only: bool = True,
) -> list[CorpusRecord]:
    records: list[CorpusRecord] = []
    with input_path.open("r", encoding="utf-8-sig", newline="") as handle:
        reader = csv.DictReader(handle)
        for index, row in enumerate(reader, start=1):
            text = _pick_first(row, "Excerpt", "excerpt").strip()
            if not text:
                continue
            license_name = _pick_first(row, "License", "license").strip()
            if open_license_only and not is_open_license(license_name):
                continue
            grade_level = _pick_first(
                row,
                "Flesch-Kincaid-Grade-Level",
                "Flesch-Kincaid Grade Level",
                "grade_level",
            ).strip()
            record = CorpusRecord(
                record_id=f"clear-{_pick_first(row, 'ID', 'id').strip() or index}",
                source="commonlit_clear",
                source_type="reading_passage",
                title=_pick_first(row, "Title", "title").strip() or f"CLEAR passage {index}",
                text=text,
                target_context=target_context,
                difficulty_band=_difficulty_from_grade_level(grade_level) or _infer_text_difficulty(text),
                license=license_name,
                metadata={
                    "author": _pick_first(row, "Author", "author").strip(),
                    "category": _pick_first(row, "Category", "category").strip(),
                    "source_url": _pick_first(row, "URL", "url").strip(),
                    "grade_level": grade_level,
                },
            )
            records.append(record)
            if limit is not None and len(records) >= limit:
                return records
    return records


def is_open_license(license_name: str) -> bool:
    normalized = license_name.strip().lower()
    if not normalized:
        return False
    if any(marker in normalized for marker in RESTRICTED_LICENSE_MARKERS):
        return False
    return any(marker in normalized for marker in OPEN_LICENSE_MARKERS)


def _pick_first(row: dict[str, str], *keys: str) -> str:
    for key in keys:
        value = row.get(key)
        if value is not None:
            return value
    return ""


def _difficulty_from_grade_level(raw_value: str) -> str | None:
    try:
        grade_level = float(raw_value)
    except ValueError:
        return None
    if grade_level < 5:
        return "foundation"
    if grade_level < 8:
        return "intermediate"
    if grade_level < 11:
        return "upper_intermediate"
    return "advanced"


def _infer_text_difficulty(text: str) -> str:
    words = len([part for part in text.split() if part.strip()])
    if words < 60:
        return "foundation"
    if words < 120:
        return "intermediate"
    if words < 220:
        return "upper_intermediate"
    return "advanced"
