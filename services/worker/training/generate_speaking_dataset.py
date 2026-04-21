from __future__ import annotations

import argparse
import json
import random
from pathlib import Path


RESEARCH_SUBJECTS = [
    "memory safe interface",
    "guided chunking system",
    "reading support tool",
    "low-load listening interface",
    "cognitive support dashboard",
]

RESEARCH_EFFECTS = [
    "reduces cognitive overload",
    "helps learners keep the main idea",
    "makes long sentences easier to handle",
    "reduces the risk of losing earlier words",
    "supports step-by-step reading",
]

SELF_INTRO_FIELDS = [
    "human computer interaction",
    "learning support design",
    "educational technology",
    "accessibility design",
    "language learning support",
]

MEETING_TOPICS = [
    "the main issue",
    "the current delay",
    "the key update",
    "the next action item",
    "the main risk",
]

DAILY_TOPICS = [
    ("at the cafe", "tea", "one small dessert"),
    ("at the station", "platform three", "the train time"),
    ("at the store", "this item", "the price"),
    ("in class", "my main question", "one short example"),
    ("at the reception desk", "my reservation", "the room number"),
]

GENERAL_TOPICS = [
    ("the short conclusion", "one supporting reason"),
    ("the main answer", "one example"),
    ("the key point", "one supporting detail"),
    ("the short version", "one next step"),
]


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", required=True)
    parser.add_argument("--count", type=int, default=1000)
    parser.add_argument("--seed", type=int, default=42)
    args = parser.parse_args()

    rng = random.Random(args.seed)
    output_path = Path(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)

    builders = [
        build_research_record,
        build_self_intro_record,
        build_meeting_record,
        build_daily_record,
        build_general_record,
    ]

    with output_path.open("w", encoding="utf-8") as handle:
        for index in range(args.count):
            builder = builders[index % len(builders)]
            record = builder(rng, index)
            handle.write(json.dumps(record, ensure_ascii=False) + "\n")


def build_research_record(rng: random.Random, index: int) -> dict[str, object]:
    subject = rng.choice(RESEARCH_SUBJECTS)
    effect = rng.choice(RESEARCH_EFFECTS)
    text = f"In this study, we propose a {subject} that {effect} during English reading."
    return {
        "task": "speaking_plan",
        "text": text,
        "language": "en",
        "target_context": "research",
        "learner_profile": "working_memory_low",
        "difficulty_focus": rng.choice(["sentence_holding", "speech_breakdown"]),
        "problem_types": ["opener_only", "short_unit", "two_step_link"],
        "output": {
            "summary": f"We propose a {subject} that {effect}.",
            "opener_options": [
                f"In this study, I present a {subject}.",
                f"Today I will talk about a {subject}.",
            ],
            "bridge_phrases": ["First,", "Then,"],
            "steps": [
                {"step": 1, "text": f"We propose a {subject}.", "purpose": "main_point"},
                {"step": 2, "text": f"It {effect} during English reading.", "purpose": "support"},
            ],
            "rescue_phrases": ["Let me say that more simply."],
        },
        "meta_id": f"research_{index:04d}",
    }


def build_self_intro_record(rng: random.Random, index: int) -> dict[str, object]:
    field = rng.choice(SELF_INTRO_FIELDS)
    text = f"I am a student working on {field}, and I want to help learners handle English with less overload."
    return {
        "task": "speaking_plan",
        "text": text,
        "language": "en",
        "target_context": "self_intro",
        "learner_profile": "working_memory_low",
        "difficulty_focus": rng.choice(["speech_breakdown", "sentence_holding", "anxiety_breakdown"]),
        "problem_types": ["opener_only", "short_unit"],
        "output": {
            "summary": f"I work on {field}.",
            "opener_options": [
                f"I work on {field}.",
                f"My topic is {field}.",
            ],
            "bridge_phrases": ["Also,", "My focus is"],
            "steps": [
                {"step": 1, "text": "I am a student.", "purpose": "opener"},
                {"step": 2, "text": f"I work on {field}.", "purpose": "main_point"},
                {"step": 3, "text": "I want to reduce overload in English learning.", "purpose": "support"},
            ],
            "rescue_phrases": ["One moment, please."],
        },
        "meta_id": f"self_intro_{index:04d}",
    }


def build_meeting_record(rng: random.Random, index: int) -> dict[str, object]:
    topic = rng.choice(MEETING_TOPICS)
    text = f"For this meeting, I will explain {topic} first and then suggest one next step."
    return {
        "task": "speaking_plan",
        "text": text,
        "language": "en",
        "target_context": "meeting",
        "learner_profile": "working_memory_low",
        "difficulty_focus": rng.choice(["sentence_holding", "anxiety_breakdown"]),
        "problem_types": ["opener_only", "two_step_link"],
        "output": {
            "summary": f"I will explain {topic} and suggest one next step.",
            "opener_options": [
                f"First, I will explain {topic}.",
                f"I will start with {topic}.",
            ],
            "bridge_phrases": ["Then,", "After that,"],
            "steps": [
                {"step": 1, "text": f"First, I will explain {topic}.", "purpose": "opener"},
                {"step": 2, "text": "Then, I will suggest one next step.", "purpose": "main_point"},
            ],
            "rescue_phrases": ["Can I say the short version first?"],
        },
        "meta_id": f"meeting_{index:04d}",
    }


def build_daily_record(rng: random.Random, index: int) -> dict[str, object]:
    place, first_need, second_need = rng.choice(DAILY_TOPICS)
    text = f"{place.capitalize()}, I want to ask for {first_need} first and then ask about {second_need}."
    return {
        "task": "speaking_plan",
        "text": text,
        "language": "en",
        "target_context": "daily",
        "learner_profile": "working_memory_low",
        "difficulty_focus": rng.choice(["sentence_holding", "speech_breakdown", "anxiety_breakdown"]),
        "problem_types": ["opener_only", "two_step_link"],
        "output": {
            "summary": f"I want to ask about {first_need} and {second_need}.",
            "opener_options": [
                f"First, I want to ask about {first_need}.",
                f"I will ask about {first_need} first.",
            ],
            "bridge_phrases": ["Then,", "Also,"],
            "steps": [
                {"step": 1, "text": f"First, I want to ask about {first_need}.", "purpose": "opener"},
                {"step": 2, "text": f"Then, I want to ask about {second_need}.", "purpose": "support"},
            ],
            "rescue_phrases": ["Please say that again slowly."],
        },
        "meta_id": f"daily_{index:04d}",
    }


def build_general_record(rng: random.Random, index: int) -> dict[str, object]:
    first_part, second_part = rng.choice(GENERAL_TOPICS)
    text = f"When I answer, I want to give {first_part} first and then add {second_part}."
    return {
        "task": "speaking_plan",
        "text": text,
        "language": "en",
        "target_context": "general",
        "learner_profile": "working_memory_low",
        "difficulty_focus": rng.choice(["sentence_holding", "speech_breakdown"]),
        "problem_types": ["opener_only", "two_step_link"],
        "output": {
            "summary": f"I will give {first_part} and {second_part}.",
            "opener_options": [
                f"First, I will give {first_part}.",
                "I will start with the short version.",
            ],
            "bridge_phrases": ["Then,", "One reason is"],
            "steps": [
                {"step": 1, "text": f"First, I will give {first_part}.", "purpose": "opener"},
                {"step": 2, "text": f"Then, I will add {second_part}.", "purpose": "support"},
            ],
            "rescue_phrases": ["Let me give the short answer first."],
        },
        "meta_id": f"general_{index:04d}",
    }


if __name__ == "__main__":
    main()
