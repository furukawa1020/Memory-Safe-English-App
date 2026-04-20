from __future__ import annotations

from dataclasses import dataclass

from app.chunking import ChunkingService
from app.models import CollapsePatternResult, CollapseSite, RESPONSE_VERSION
from app.text_analysis import estimate_segment_load


@dataclass(slots=True)
class CollapsePatternService:
    chunking_service: ChunkingService

    def analyze(
        self,
        text: str,
        *,
        language: str = "en",
        session_events: list[dict[str, str | int | float]] | None = None,
    ) -> CollapsePatternResult:
        chunking = self.chunking_service.chunk_text(text=text, language=language)
        chunks = chunking.chunks
        events = session_events or []
        mode_signals = _collect_mode_signals(events)

        sites: list[CollapseSite] = []
        for chunk in chunks:
            chunk_events = [event for event in events if event.get("chunk_order") == chunk.order]
            repeated = sum(1 for event in chunk_events if event.get("event_type") == "repeat")
            support_open = sum(1 for event in chunk_events if event.get("event_type") == "support_open")
            long_pause = sum(1 for event in chunk_events if event.get("event_type") == "long_pause")
            stop_count = repeated + support_open + long_pause
            if stop_count == 0 and estimate_segment_load(chunk.text) < 7:
                continue

            risk_level = _classify_risk(stop_count, estimate_segment_load(chunk.text))
            sites.append(
                CollapseSite(
                    chunk_order=chunk.order,
                    text=chunk.text,
                    role=chunk.role,
                    risk_level=risk_level,
                    stop_count=stop_count,
                    reasons=_build_reasons(repeated, support_open, long_pause, chunk.text),
                    recommendation=_build_recommendation(risk_level, chunk.role),
                )
            )

        return CollapsePatternResult(
            version=RESPONSE_VERSION,
            language=language,
            summary=chunking.summary,
            dominant_pattern=_dominant_pattern(sites),
            likely_mode=_likely_mode(mode_signals),
            mode_signals=mode_signals,
            sites=sites,
        )


def _collect_mode_signals(events: list[dict[str, str | int | float]]) -> dict[str, int]:
    mapping = {
        "repeat": "reading",
        "support_open": "reading",
        "long_pause": "reading",
        "audio_restart": "listening",
        "audio_pause": "listening",
        "speed_down": "listening",
        "speech_restart": "speaking",
        "long_silence": "speaking",
        "template_open": "speaking",
    }
    signals = {"reading": 0, "listening": 0, "speaking": 0}
    for event in events:
        event_type = str(event.get("event_type", ""))
        mode = mapping.get(event_type)
        if mode is not None:
            signals[mode] += 1
    return signals


def _classify_risk(stop_count: int, load_score: int) -> str:
    score = stop_count + (2 if load_score >= 9 else 1 if load_score >= 6 else 0)
    if score >= 4:
        return "high"
    if score >= 2:
        return "medium"
    return "low"


def _build_reasons(repeated: int, support_open: int, long_pause: int, text: str) -> list[str]:
    reasons: list[str] = []
    if repeated:
        reasons.append("the learner repeated this part several times")
    if support_open:
        reasons.append("the learner opened support while processing this part")
    if long_pause:
        reasons.append("the learner paused for a long time on this part")
    if estimate_segment_load(text) >= 7:
        reasons.append("the segment itself looks linguistically heavy")
    return reasons or ["this part may still need lighter presentation"]


def _build_recommendation(risk_level: str, role: str) -> str:
    if risk_level == "high":
        return "show this chunk alone first and delay surrounding support"
    if role == "modifier":
        return "collapse this modifier until the core meaning is stable"
    return "keep this chunk visually anchored and reduce nearby detail"


def _dominant_pattern(sites: list[CollapseSite]) -> str:
    if not sites:
        return "no clear collapse pattern detected"
    high_count = sum(1 for site in sites if site.risk_level == "high")
    modifier_count = sum(1 for site in sites if site.role == "modifier")
    if high_count >= 2:
        return "multiple high-risk points suggest strong memory overload during integration"
    if modifier_count >= len(sites) / 2:
        return "collapse often happens around modifiers and supporting detail"
    return "collapse tends to happen when the main chunk becomes heavy or repeated"


def _likely_mode(mode_signals: dict[str, int]) -> str:
    strongest_mode = max(mode_signals, key=mode_signals.get)
    if mode_signals[strongest_mode] == 0:
        return "mixed"
    tied_modes = [mode for mode, score in mode_signals.items() if score == mode_signals[strongest_mode]]
    if len(tied_modes) > 1:
        return "mixed"
    return strongest_mode
