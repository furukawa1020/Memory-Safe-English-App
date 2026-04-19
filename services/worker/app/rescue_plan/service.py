from __future__ import annotations

from dataclasses import dataclass

from app.chunking import ChunkingService
from app.models import RESPONSE_VERSION, RescuePhrase, RescuePlanResult
from app.text_analysis import estimate_segment_load


@dataclass(slots=True)
class RescuePlanService:
    chunking_service: ChunkingService

    def build(self, text: str, language: str = "en") -> RescuePlanResult:
        chunking = self.chunking_service.chunk_text(text=text, language=language)
        chunks = chunking.chunks
        total_load = sum(estimate_segment_load(chunk.text) for chunk in chunks)
        overload_level = _classify_overload_level(total_load, len(chunks))

        return RescuePlanResult(
            version=RESPONSE_VERSION,
            language=language,
            summary=chunking.summary,
            overload_level=overload_level,
            primary_strategy=_primary_strategy(overload_level),
            phrases=_build_rescue_phrases(overload_level),
        )


def _classify_overload_level(total_load: int, chunk_count: int) -> str:
    score = total_load + chunk_count
    if score >= 20:
        return "high"
    if score >= 11:
        return "medium"
    return "low"


def _primary_strategy(overload_level: str) -> str:
    if overload_level == "high":
        return "slow the speaker down and ask for a shorter restatement"
    if overload_level == "medium":
        return "ask for the main point before continuing"
    return "use a short confirmation phrase and keep the exchange moving"


def _build_rescue_phrases(overload_level: str) -> list[RescuePhrase]:
    base = [
        RescuePhrase(
            category="slow_down",
            phrase_en="Please say that more slowly.",
            phrase_ja="もう少しゆっくり言ってください。",
            use_when="Use this when the incoming speech is too fast to hold.",
            priority=1 if overload_level == "high" else 2,
        ),
        RescuePhrase(
            category="shorter",
            phrase_en="Can you say it in a shorter way?",
            phrase_ja="もう少し短く言ってもらえますか。",
            use_when="Use this when the sentence is too dense to keep in mind.",
            priority=1 if overload_level in {"high", "medium"} else 3,
        ),
        RescuePhrase(
            category="main_point",
            phrase_en="What is the main point?",
            phrase_ja="要点は何ですか。",
            use_when="Use this when you need only the core meaning first.",
            priority=2,
        ),
        RescuePhrase(
            category="buy_time",
            phrase_en="One moment, please.",
            phrase_ja="少し待ってください。",
            use_when="Use this when you need a short pause before answering.",
            priority=3,
        ),
        RescuePhrase(
            category="confirm",
            phrase_en="Do you mean ...?",
            phrase_ja="つまり ... という意味ですか。",
            use_when="Use this when you think you caught the idea and want to confirm it.",
            priority=4,
        ),
    ]
    return sorted(base, key=lambda phrase: phrase.priority)
