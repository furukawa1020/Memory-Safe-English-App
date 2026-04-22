from __future__ import annotations

import argparse
import json
from pathlib import Path

from app.assessment import AssessmentService
from app.chunking import ChunkingService
from app.listening_plan import ListeningPlanService
from app.practice_set import PracticeSetService
from app.reader_plan import ReaderPlanService
from app.rescue_plan import RescuePlanService
from app.speaking_plan import SpeakingPlanService
from training.public_corpora import CorpusRecord, read_corpus_jsonl


DEFAULT_TASKS = (
    "chunking",
    "summary",
    "reader_plan",
    "listening_plan",
    "speaking_plan",
    "rescue_plan",
    "practice_set",
)


def main() -> None:
    parser = argparse.ArgumentParser(
        description="canonical corpus JSONL から WM 特化の raw training JSONL を作ります。",
    )
    parser.add_argument("--input", required=True, help="canonical corpus JSONL")
    parser.add_argument("--output", required=True, help="raw training JSONL の出力先")
    parser.add_argument(
        "--tasks",
        default=",".join(DEFAULT_TASKS),
        help="生成する task。カンマ区切り。",
    )
    parser.add_argument("--limit", type=int, default=None, help="先頭から処理する corpus 件数")
    args = parser.parse_args()

    tasks = tuple(part.strip() for part in args.tasks.split(",") if part.strip())
    unsupported = sorted(set(tasks) - set(DEFAULT_TASKS))
    if unsupported:
        raise ValueError(f"unsupported tasks: {', '.join(unsupported)}")

    records = read_corpus_jsonl(Path(args.input))
    if args.limit is not None:
        records = records[: args.limit]

    builder = WMTrainingBuilder()
    output_path = Path(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    written = 0
    with output_path.open("w", encoding="utf-8") as handle:
        for record in records:
            for training_record in builder.build_records(record, tasks=tasks):
                handle.write(json.dumps(training_record, ensure_ascii=False) + "\n")
                written += 1
    print(f"wrote {written} training records to {output_path}")


class WMTrainingBuilder:
    def __init__(self) -> None:
        chunking_service = ChunkingService()
        self._chunking_service = chunking_service
        self._reader_plan_service = ReaderPlanService(chunking_service=chunking_service)
        self._listening_plan_service = ListeningPlanService(chunking_service=chunking_service)
        self._speaking_plan_service = SpeakingPlanService(chunking_service=chunking_service)
        self._rescue_plan_service = RescuePlanService(chunking_service=chunking_service)
        self._assessment_service = AssessmentService()
        self._practice_set_service = PracticeSetService(
            reader_plan_service=self._reader_plan_service,
            listening_plan_service=self._listening_plan_service,
            speaking_plan_service=self._speaking_plan_service,
            rescue_plan_service=self._rescue_plan_service,
            assessment_service=self._assessment_service,
        )

    def build_records(self, corpus_record: CorpusRecord, *, tasks: tuple[str, ...]) -> list[dict[str, object]]:
        text = corpus_record.text.strip()
        if not text:
            return []

        target_context = corpus_record.target_context
        language = corpus_record.language
        chunking_result = self._chunking_service.chunk_text(text=text, language=language)

        outputs: list[dict[str, object]] = []
        for task in tasks:
            output = self._build_output(
                task,
                corpus_record=corpus_record,
                text=text,
                language=language,
                target_context=target_context,
                chunking_result=chunking_result,
            )
            if output is None:
                continue
            outputs.append(
                {
                    "task": task,
                    "text": text,
                    "language": language,
                    "target_context": target_context,
                    "learner_profile": "working_memory_low",
                    "difficulty_focus": _difficulty_focus_for_task(task),
                    "problem_types": _problem_types_for_task(task),
                    "source_record_id": corpus_record.record_id,
                    "source": corpus_record.source,
                    "difficulty_band": corpus_record.difficulty_band,
                    "output": output,
                }
            )
        return outputs

    def _build_output(
        self,
        task: str,
        *,
        corpus_record: CorpusRecord,
        text: str,
        language: str,
        target_context: str,
        chunking_result,
    ) -> dict[str, object] | None:
        if task == "chunking":
            return {"segments": [chunk.text for chunk in chunking_result.chunks]}
        if task == "summary":
            return {"summary": chunking_result.summary}
        if task == "reader_plan":
            return _strip_common_fields(
                self._reader_plan_service.build(
                    text=text,
                    language=language,
                    target_context=target_context,
                ).to_dict()
            )
        if task == "listening_plan":
            return _strip_common_fields(
                self._listening_plan_service.build(
                    text=text,
                    language=language,
                    target_context=target_context,
                ).to_dict()
            )
        if task == "speaking_plan":
            return _strip_common_fields(
                self._speaking_plan_service.build(
                    text=text,
                    language=language,
                    target_context=target_context,
                ).to_dict()
            )
        if task == "rescue_plan":
            return _strip_common_fields(
                self._rescue_plan_service.build(
                    text=text,
                    language=language,
                    target_context=target_context,
                ).to_dict()
            )
        if task == "practice_set":
            return _strip_common_fields(
                self._practice_set_service.build(
                    text=text,
                    language=language,
                    target_context=target_context,
                    self_reported_difficulties=[_difficulty_focus_for_task("speaking_plan")],
                    fatigue_level=_fatigue_level_for_band(corpus_record.difficulty_band),
                ).to_dict()
            )
        return None


def _strip_common_fields(payload: dict[str, object]) -> dict[str, object]:
    normalized = dict(payload)
    normalized.pop("version", None)
    normalized.pop("language", None)
    return normalized


def _difficulty_focus_for_task(task: str) -> str:
    mapping = {
        "chunking": "sentence_integration",
        "summary": "gist_retention",
        "reader_plan": "sentence_integration",
        "listening_plan": "audio_tracking",
        "speaking_plan": "sentence_holding",
        "rescue_plan": "overload_recovery",
        "practice_set": "mixed_support",
    }
    return mapping.get(task, "")


def _problem_types_for_task(task: str) -> list[str]:
    mapping = {
        "reader_plan": ["core_lock", "support_attach"],
        "listening_plan": ["pause_recall", "meaning_hold"],
        "speaking_plan": ["opener_only", "short_unit", "two_step_link"],
        "rescue_plan": ["rescue_phrase"],
        "practice_set": ["core_lock", "pause_recall", "opener_only", "rescue_phrase"],
    }
    return mapping.get(task, [])


def _fatigue_level_for_band(difficulty_band: str) -> str:
    if difficulty_band in {"advanced", "upper_intermediate"}:
        return "medium"
    return "low"


if __name__ == "__main__":
    main()
