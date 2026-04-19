from app.assessment import AssessmentService


def test_assessment_returns_recommendations_for_high_load_profile() -> None:
    service = AssessmentService()

    result = service.assess(
        "In this study, we propose a memory safe interface that reduces overload during reading.",
        language="en",
        target_context="research",
        self_reported_difficulties=["sentence_integration", "audio_tracking", "speech_breakdown"],
        fatigue_level="high",
    )

    assert result.profile_label == "研究説明"
    assert result.recommended_reader_mode in {"chunk", "assisted"}
    assert result.recommended_listening_mode in {"sentence_pause", "chunk_pause"}
    assert result.recommended_speaking_mode in {"short_steps", "template_short_steps"}
    assert result.reasons
