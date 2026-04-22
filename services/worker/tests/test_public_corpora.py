from __future__ import annotations

import json
from pathlib import Path

from training.build_wm_training_corpus import WMTrainingBuilder
from training.public_corpora import (
    CorpusRecord,
    is_open_license,
    load_clear_records,
    load_squad_records,
    write_corpus_jsonl,
)


def test_is_open_license_rejects_nc_license() -> None:
    assert is_open_license("CC BY-SA 4.0")
    assert not is_open_license("CC BY-NC 4.0")


def test_load_squad_records_extracts_unique_paragraphs(tmp_path: Path) -> None:
    input_path = tmp_path / "squad.json"
    input_path.write_text(
        json.dumps(
            {
                "data": [
                    {
                        "title": "Memory",
                        "paragraphs": [
                            {
                                "context": "Memory-safe interfaces reduce overload during reading.",
                                "qas": [
                                    {"id": "q1", "question": "What do interfaces reduce?", "answers": [], "is_impossible": False},
                                    {"id": "q2", "question": "What task?", "answers": [], "is_impossible": True},
                                ],
                            }
                        ],
                    }
                ]
            }
        ),
        encoding="utf-8",
    )

    records = load_squad_records(input_path, target_context="general")

    assert len(records) == 1
    assert records[0].source == "squad_v2"
    assert records[0].metadata["question_count"] == 2
    assert records[0].metadata["impossible_count"] == 1


def test_load_clear_records_filters_restricted_license(tmp_path: Path) -> None:
    input_path = tmp_path / "clear.csv"
    input_path.write_text(
        "\n".join(
            [
                "ID,Title,Excerpt,License,Flesch-Kincaid-Grade-Level,Author",
                '1,Open passage,"This passage is usable for training.",CC BY-SA 4.0,7.2,Author A',
                '2,Restricted passage,"This passage should be skipped.",CC BY-NC 4.0,7.2,Author B',
            ]
        ),
        encoding="utf-8",
    )

    records = load_clear_records(input_path, target_context="general", open_license_only=True)

    assert len(records) == 1
    assert records[0].title == "Open passage"
    assert records[0].difficulty_band == "intermediate"


def test_wm_training_builder_emits_speaking_and_practice_set() -> None:
    builder = WMTrainingBuilder()
    corpus_record = CorpusRecord(
        record_id="sample-1",
        source="unit_test",
        source_type="reading_passage",
        title="Memory-safe reading",
        text="In this study, we propose a memory safe interface that reduces overload during reading.",
        target_context="research",
        difficulty_band="upper_intermediate",
    )

    records = builder.build_records(corpus_record, tasks=("chunking", "speaking_plan", "practice_set"))
    by_task = {record["task"]: record for record in records}

    assert by_task["chunking"]["output"]["segments"]
    assert by_task["speaking_plan"]["problem_types"] == ["opener_only", "short_unit", "two_step_link"]
    assert by_task["speaking_plan"]["output"]["steps"]
    assert by_task["practice_set"]["output"]["sections"]


def test_write_corpus_jsonl_writes_records(tmp_path: Path) -> None:
    output_path = tmp_path / "corpus.jsonl"
    count = write_corpus_jsonl(
        [
            CorpusRecord(
                record_id="sample-1",
                source="unit_test",
                source_type="reading_passage",
                title="Sample",
                text="Simple text for testing.",
            )
        ],
        output_path,
    )

    assert count == 1
    lines = output_path.read_text(encoding="utf-8").strip().splitlines()
    assert len(lines) == 1
