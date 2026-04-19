from __future__ import annotations

from dataclasses import dataclass

from app.context_profile import resolve_context_profile
from app.models import AssessmentProfileResult, RESPONSE_VERSION
from app.text_analysis import estimate_segment_load


@dataclass(slots=True)
class AssessmentService:
    def assess(
        self,
        text: str,
        *,
        language: str = "en",
        target_context: str = "general",
        self_reported_difficulties: list[str] | None = None,
        fatigue_level: str = "unknown",
    ) -> AssessmentProfileResult:
        profile = resolve_context_profile(target_context)
        normalized_difficulties = self_reported_difficulties or []
        text_load = estimate_segment_load(text)

        reading_load_score = _base_score(text_load, normalized_difficulties, "reading")
        listening_load_score = _base_score(text_load, normalized_difficulties, "listening")
        speaking_load_score = _base_score(text_load, normalized_difficulties, "speaking")

        fatigue_adjustment = {"unknown": 0, "low": 0, "medium": 1, "high": 2}[fatigue_level]
        reading_load_score = min(5, reading_load_score + fatigue_adjustment)
        listening_load_score = min(5, listening_load_score + fatigue_adjustment)
        speaking_load_score = min(5, speaking_load_score + fatigue_adjustment)

        recommended_reader_mode = _recommend_reader_mode(reading_load_score)
        recommended_listening_mode = _recommend_listening_mode(listening_load_score)
        recommended_speaking_mode = _recommend_speaking_mode(speaking_load_score)

        return AssessmentProfileResult(
            version=RESPONSE_VERSION,
            language=language,
            target_context=target_context,
            profile_label=profile.label_ja,
            notice="This is a UI optimization estimate, not a medical diagnosis.",
            reading_load_score=reading_load_score,
            listening_load_score=listening_load_score,
            speaking_load_score=speaking_load_score,
            recommended_reader_mode=recommended_reader_mode,
            recommended_listening_mode=recommended_listening_mode,
            recommended_speaking_mode=recommended_speaking_mode,
            recommended_display_density=_recommend_display_density(reading_load_score),
            recommended_pause_frequency=_recommend_pause_frequency(listening_load_score),
            reasons=_build_reasons(normalized_difficulties, fatigue_level, profile.label_ja),
        )


def _base_score(text_load: int, difficulties: list[str], mode: str) -> int:
    score = 1
    if text_load >= 8:
        score += 1
    if text_load >= 12:
        score += 1

    difficulty_map = {
        "reading": {"sentence_integration", "drops_previous_words", "long_text"},
        "listening": {"audio_tracking", "drops_previous_words", "fast_audio"},
        "speaking": {"sentence_holding", "speech_breakdown", "anxiety_breakdown"},
    }
    score += sum(1 for difficulty in difficulties if difficulty in difficulty_map[mode])
    return min(5, score)


def _recommend_reader_mode(score: int) -> str:
    if score >= 4:
        return "assisted"
    if score >= 3:
        return "chunk"
    return "normal"


def _recommend_listening_mode(score: int) -> str:
    if score >= 4:
        return "chunk_pause"
    if score >= 3:
        return "sentence_pause"
    return "continuous"


def _recommend_speaking_mode(score: int) -> str:
    if score >= 4:
        return "template_short_steps"
    if score >= 3:
        return "short_steps"
    return "free_short"


def _recommend_display_density(score: int) -> str:
    if score >= 4:
        return "minimal"
    if score >= 3:
        return "reduced"
    return "standard"


def _recommend_pause_frequency(score: int) -> str:
    if score >= 4:
        return "high"
    if score >= 3:
        return "medium"
    return "low"


def _build_reasons(difficulties: list[str], fatigue_level: str, context_label: str) -> list[str]:
    reasons = [f"Target context: {context_label}."]
    if difficulties:
        reasons.append(f"Reported difficulties: {', '.join(difficulties)}.")
    if fatigue_level != "unknown":
        reasons.append(f"Fatigue level considered: {fatigue_level}.")
    if len(reasons) == 1:
        reasons.append("No self-reported difficulties were provided, so the estimate relies on text load only.")
    return reasons
