from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class ContextProfile:
    key: str
    label_ja: str
    listening_focus_prompt: str
    speaking_opener_prefix: str
    rescue_priority: str


_PROFILES: dict[str, ContextProfile] = {
    "general": ContextProfile(
        key="general",
        label_ja="汎用英語",
        listening_focus_prompt="Keep only the core meaning.",
        speaking_opener_prefix="The main point is",
        rescue_priority="main_point",
    ),
    "self_intro": ContextProfile(
        key="self_intro",
        label_ja="自己紹介",
        listening_focus_prompt="Keep the person's role and main point.",
        speaking_opener_prefix="Let me introduce myself",
        rescue_priority="buy_time",
    ),
    "research": ContextProfile(
        key="research",
        label_ja="研究説明",
        listening_focus_prompt="Keep the claim, method, or result first.",
        speaking_opener_prefix="In this study",
        rescue_priority="main_point",
    ),
    "meeting": ContextProfile(
        key="meeting",
        label_ja="会議",
        listening_focus_prompt="Keep the decision or action item first.",
        speaking_opener_prefix="The main update is",
        rescue_priority="shorter",
    ),
    "daily": ContextProfile(
        key="daily",
        label_ja="日常会話",
        listening_focus_prompt="Keep only the everyday meaning and move on.",
        speaking_opener_prefix="Basically",
        rescue_priority="slow_down",
    ),
}


def resolve_context_profile(target_context: str) -> ContextProfile:
    return _PROFILES.get(target_context, _PROFILES["general"])
