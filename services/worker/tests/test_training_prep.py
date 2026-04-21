import json

from training.prepare_seq2seq_data import normalize_record


def test_normalize_record_builds_prompt_and_target() -> None:
    result = normalize_record(
        {
            "task": "chunking",
            "text": "We propose a memory safe interface.",
            "language": "en",
            "target_context": "research",
            "output": {"segments": ["We propose", "a memory safe interface."]},
        }
    )

    assert "Task: chunking" in result["prompt"]
    assert "Target context: research" in result["prompt"]
    assert json.loads(result["target"])["segments"][0] == "We propose"
