from __future__ import annotations

from dataclasses import dataclass


VALID_TARGET_CONTEXTS = {"general", "self_intro", "research", "meeting", "daily"}


@dataclass(frozen=True, slots=True)
class ContextProfile:
    key: str
    label_en: str
    label_ja: str
    speaking_opener_prefix: str
    listening_focus_prompt: str
    rescue_priority: str


CONTEXT_PROFILES = {
    "general": ContextProfile(
        key="general",
        label_en="General English",
        label_ja="汎用英語",
        speaking_opener_prefix="The main point is",
        listening_focus_prompt="Catch the main idea first.",
        rescue_priority="main_point",
    ),
    "self_intro": ContextProfile(
        key="self_intro",
        label_en="Self Introduction",
        label_ja="自己紹介",
        speaking_opener_prefix="Let me introduce myself",
        listening_focus_prompt="Catch the personal facts first.",
        rescue_priority="confirm",
    ),
    "research": ContextProfile(
        key="research",
        label_en="Research",
        label_ja="研究説明",
        speaking_opener_prefix="In this study",
        listening_focus_prompt="Catch the claim or method first.",
        rescue_priority="main_point",
    ),
    "meeting": ContextProfile(
        key="meeting",
        label_en="Meeting",
        label_ja="会議",
        speaking_opener_prefix="My main point is",
        listening_focus_prompt="Catch the decision or request first.",
        rescue_priority="shorter",
    ),
    "daily": ContextProfile(
        key="daily",
        label_en="Daily Conversation",
        label_ja="日常会話",
        speaking_opener_prefix="I want to say",
        listening_focus_prompt="Catch the topic first, then the detail.",
        rescue_priority="slow_down",
    ),
}


def resolve_context_profile(target_context: str) -> ContextProfile:
    return CONTEXT_PROFILES.get(target_context, CONTEXT_PROFILES["general"])
