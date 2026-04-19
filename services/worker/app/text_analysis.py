from __future__ import annotations

import re


CLAUSE_BREAKS = re.compile(r"\s*(,|;|:|\band\b|\bbut\b|\bthat\b|\bwhich\b)\s*", re.IGNORECASE)
VERB_HINTS = {
    "am",
    "is",
    "are",
    "was",
    "were",
    "be",
    "been",
    "being",
    "do",
    "does",
    "did",
    "have",
    "has",
    "had",
    "use",
    "uses",
    "used",
    "need",
    "needs",
    "needed",
    "make",
    "makes",
    "made",
    "propose",
    "proposes",
    "proposed",
    "show",
    "shows",
    "showed",
    "study",
    "studies",
    "studied",
    "support",
    "supports",
    "supported",
    "reduce",
    "reduces",
    "reduced",
    "help",
    "helps",
    "helped",
}
LEADING_MODIFIER_PREFIXES = ("to ", "for ", "with ", "in ", "on ", "at ", "by ", "during ", "after ", "before ")


def normalize_text(text: str) -> str:
    return " ".join(text.split())


def segment_text(text: str) -> list[str]:
    rough_parts = [part.strip(" ,;:") for part in CLAUSE_BREAKS.split(text)]
    return [part for part in rough_parts if part and part not in {",", ";", ":"}]


def looks_core(segment: str) -> bool:
    return any(word.lower().strip(".,!?") in VERB_HINTS for word in segment.split())


def infer_role(segment: str, index: int) -> str:
    lowered = segment.lower()
    if index == 0 and not looks_core(segment):
        return "modifier"
    if lowered.startswith(LEADING_MODIFIER_PREFIXES):
        return "modifier"
    if looks_core(segment):
        return "core"
    return "support"
