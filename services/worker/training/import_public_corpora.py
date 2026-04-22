from __future__ import annotations

import argparse
from pathlib import Path

from training.public_corpora import (
    load_clear_records,
    load_squad_records,
    write_corpus_jsonl,
)


def main() -> None:
    parser = argparse.ArgumentParser(
        description="公開コーパスを canonical corpus JSONL へ変換します。",
    )
    subparsers = parser.add_subparsers(dest="dataset", required=True)

    squad_parser = subparsers.add_parser("squad", help="SQuAD v2 JSON を取り込む")
    _add_common_arguments(squad_parser)

    clear_parser = subparsers.add_parser("clear", help="CommonLit CLEAR CSV を取り込む")
    _add_common_arguments(clear_parser)
    clear_parser.add_argument(
        "--include-restricted-license",
        action="store_true",
        help="ライセンスが強く制限される行も含めます。既定では open license のみです。",
    )

    args = parser.parse_args()
    input_path = Path(args.input)
    output_path = Path(args.output)

    if args.dataset == "squad":
        records = load_squad_records(
            input_path,
            target_context=args.target_context,
            limit=args.limit,
        )
    else:
        records = load_clear_records(
            input_path,
            target_context=args.target_context,
            limit=args.limit,
            open_license_only=not args.include_restricted_license,
        )

    count = write_corpus_jsonl(records, output_path)
    print(f"wrote {count} corpus records to {output_path}")


def _add_common_arguments(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--input", required=True, help="ダウンロード済みデータセットのパス")
    parser.add_argument("--output", required=True, help="canonical corpus JSONL の出力先")
    parser.add_argument(
        "--target-context",
        default="general",
        choices=["general", "research", "self_intro", "meeting", "daily"],
        help="このコーパスを主にどの学習文脈で使うか",
    )
    parser.add_argument("--limit", type=int, default=None, help="先頭から取り込む件数の上限")


if __name__ == "__main__":
    main()
