import json

from training.prepare_seq2seq_data import normalize_record


def test_normalize_record_includes_speaking_metadata_in_prompt() -> None:
    result = normalize_record(
        {
            "task": "speaking_plan",
            "text": "I will explain the issue and then suggest one next step.",
            "language": "en",
            "target_context": "meeting",
            "learner_profile": "working_memory_low",
            "difficulty_focus": "sentence_holding",
            "problem_types": ["opener_only", "two_step_link"],
            "output": {
                "opener_options": ["First, I will explain the issue."],
                "steps": [{"step": 1, "text": "First, I will explain the issue.", "purpose": "opener"}],
            },
        }
    )

    assert "Learner profile: working_memory_low" in result["prompt"]
    assert "Difficulty focus: sentence_holding" in result["prompt"]
    assert "Problem types: opener_only, two_step_link" in result["prompt"]
    assert json.loads(result["target"])["opener_options"][0].startswith("First,")
