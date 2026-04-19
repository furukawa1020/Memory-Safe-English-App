from __future__ import annotations

import re


SENTENCE_BREAKS = re.compile(r"(?<=[.!?])\s+")
CLAUSE_BREAKS = re.compile(r"\s*(,|;|:|\band\b|\bbut\b|\bthat\b|\bwhich\b|\bwhile\b|\balthough\b|\bbecause\b)\s*", re.IGNORECASE)
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
SOFT_SPLIT_PREFIXES = {
    "to",
    "for",
    "with",
    "in",
    "on",
    "at",
    "by",
    "during",
    "after",
    "before",
    "because",
    "while",
    "that",
    "which",
}


def normalize_text(text: str) -> str:
    return " ".join(text.split())


def segment_text(text: str) -> list[str]:
    sentences = [sentence.strip() for sentence in SENTENCE_BREAKS.split(text) if sentence.strip()]
    segments: list[str] = []
    for sentence in sentences:
        rough_parts = [part.strip(" ,;:") for part in CLAUSE_BREAKS.split(sentence)]
        sentence_segments = [part for part in rough_parts if part and part not in {",", ";", ":"}]
        if sentence_segments:
            segments.extend(sentence_segments)
        else:
            segments.append(sentence)
    return segments


def split_long_segment(segment: str, max_words: int) -> list[str]:
    words = segment.split()
    if len(words) <= max_words:
        return [segment]

    refined: list[str] = []
    start = 0
    while start < len(words):
        end = min(start + max_words, len(words))
        if end < len(words):
            pivot = _find_soft_split(words, start, end)
            if pivot > start:
                end = pivot
        refined.append(" ".join(words[start:end]))
        start = end
    return refined


def summarize_segments(segments: list[str], *, max_segments: int = 2) -> str:
    if not segments:
        return ""
    return " / ".join(segments[:max_segments])


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


def _find_soft_split(words: list[str], start: int, end: int) -> int:
    for index in range(end - 1, start, -1):
        if words[index].lower().strip(".,!?") in SOFT_SPLIT_PREFIXES:
            return index
    return end
