# Worker 学習基盤

このディレクトリは、worker の `chunking / summary / practice-set` 系を自分で fine-tune するための土台です。

## 推奨モデル

今の worker 構成に最もつなぎやすい標準モデルは `google/flan-t5-large` です。

- `google/flan-t5-large`
  まず最初に試す標準候補
- `google/flan-t5-xl`
  さらに強いが、学習コストも上がる
- `Qwen/Qwen2.5-7B-Instruct`
  かなり強いが、worker 側を causal LM 前提に寄せる追加改修が必要

## データ形式

学習元データは JSONL を想定しています。

```json
{
  "task": "chunking",
  "text": "In this study, we propose a memory safe interface that reduces overload during reading.",
  "language": "en",
  "target_context": "research",
  "output": {
    "segments": [
      "In this study, we propose a",
      "memory safe interface",
      "that reduces overload during reading."
    ]
  }
}
```

`task` は次を想定しています。

- `chunking`
- `summary`
- `reader_plan`
- `listening_plan`
- `speaking_plan`
- `rescue_plan`

まずは `chunking` と `summary` から始めるのが安全です。

## 手順

1. optional dependency を入れる

```bash
pip install .[training]
```

2. JSONL を学習用 prompt/target 形式に変換する

```bash
python training/prepare_seq2seq_data.py ^
  --input data/train_raw.jsonl ^
  --output data/train_prepared.jsonl
```

3. LoRA で学習する

```bash
python training/train_seq2seq_lora.py ^
  --train-file data/train_prepared.jsonl ^
  --eval-file data/eval_prepared.jsonl ^
  --model-name google/flan-t5-large ^
  --output-dir checkpoints/flan-t5-large-lora
```

4. 学習済み adapter を worker の transformer backend で使う

```powershell
$env:WORKER_NLP_BACKEND='transformer'
$env:WORKER_TRANSFORMER_MODEL='checkpoints/flan-t5-large-lora'
python -m app.server
```

## 学習方針

- 最初は `chunking` と `summary` に絞る
- JSON 以外の自由文を target に混ぜすぎない
- 1 サンプル 1 意図にする
- `research / meeting / self_intro / daily` を混ぜて context 一般化を作る
- validation では JSON parse 成功率と、期待キーの一致率も見る
