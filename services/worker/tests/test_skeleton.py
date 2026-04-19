from app.skeleton import SkeletonService


def test_skeleton_extracts_core_parts() -> None:
    service = SkeletonService()

    result = service.extract("In this study, we propose a memory safe interface for English reading.")

    assert result.parts
    assert any(part.role == "core" for part in result.parts)
    assert result.summary


def test_skeleton_falls_back_when_only_modifiers_exist() -> None:
    service = SkeletonService()

    result = service.extract("For English reading support.")

    assert len(result.parts) == 1
    assert result.parts[0].text == "For English reading support."
