from __future__ import annotations

from dataclasses import dataclass

from app.models import RESPONSE_VERSION, SkeletonPart, SkeletonResult
from app.text_analysis import infer_role, looks_core, normalize_text, segment_text


@dataclass(slots=True)
class SkeletonService:
    def extract(self, text: str, language: str = "en") -> SkeletonResult:
        normalized = normalize_text(text)
        if not normalized:
            return SkeletonResult(version=RESPONSE_VERSION, language=language, parts=[], summary="")

        segments = segment_text(normalized)
        if not segments:
            segments = [normalized]

        parts = []
        for index, segment in enumerate(segments):
            role = infer_role(segment, index)
            if role == "modifier" and not looks_core(segment):
                continue
            emphasis = 2 if role == "core" else 1
            parts.append(
                SkeletonPart(
                    order=len(parts) + 1,
                    text=segment,
                    role=role,
                    emphasis=emphasis,
                )
            )

        if not parts:
            parts.append(SkeletonPart(order=1, text=segments[0], role="support", emphasis=1))

        summary = " -> ".join(part.text for part in parts[:3])
        return SkeletonResult(version=RESPONSE_VERSION, language=language, parts=parts, summary=summary)
