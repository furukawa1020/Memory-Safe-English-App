# Training ワークフロー

このディレクトリは、worker の `chunking / summary / reader_plan / listening_plan / speaking_plan / rescue_plan / practice_set` を fine-tune したり、教師データを整形したりするための場所です。

## 方針

- まずは `公開データセット` を土台に使う
- その上に `ワーキングメモリ弱めの学習者向けラベル` を重ねる
- 雑な自動 augmentation を主力にしない
- `speaking_plan` は `opener_only / short_unit / two_step_link` を特に重視する

## 使うデータの考え方

### 1. 土台データ

- `SQuAD v2`
  - 読解 passage と QA の土台
  - 公式配布は CC BY-SA 4.0
- `CommonLit CLEAR`
  - 難度つきの reading passage
  - 行ごとに license を確認する必要あり

TOEIC そのものの問題文は著作権・利用条件の制約が強いため、ここでは `TOEIC 風の難度帯` を公開データから再構成する前提にしています。

### 2. WM 特化ラベル

- `sentence_integration`
- `audio_tracking`
- `sentence_holding`
- `overload_recovery`

### 3. 問題タイプ

- `core_lock`
- `support_attach`
- `pause_recall`
- `meaning_hold`
- `opener_only`
- `short_unit`
- `two_step_link`
- `rescue_phrase`

## 公開コーパスの取り込み

公開データセットを canonical corpus JSONL へ変換します。

### SQuAD v2

```bash
python training/import_public_corpora.py squad ^
  --input data/train-v2.0.json ^
  --output training/artifacts/squad_corpus.jsonl ^
  --target-context general
```

### CommonLit CLEAR

```bash
python training/import_public_corpora.py clear ^
  --input data/CLEARCorpus.csv ^
  --output training/artifacts/clear_corpus.jsonl ^
  --target-context general
```

既定では `open license` と判定できる行だけを取り込みます。制限付きライセンスも含めたい場合だけ `--include-restricted-license` を付けてください。

## WM 特化の raw training JSONL 生成

canonical corpus JSONL から worker の teacher output を使って raw training JSONL を作ります。

```bash
python training/build_wm_training_corpus.py ^
  --input training/artifacts/squad_corpus.jsonl ^
  --output training/artifacts/squad_wm_raw.jsonl ^
  --tasks chunking,summary,reader_plan,listening_plan,speaking_plan,rescue_plan,practice_set
```

この段階で、各レコードは次のような形になります。

```json
{
  "task": "speaking_plan",
  "text": "In this study, we propose a memory safe interface that reduces overload during reading.",
  "language": "en",
  "target_context": "general",
  "learner_profile": "working_memory_low",
  "difficulty_focus": "sentence_holding",
  "problem_types": ["opener_only", "short_unit", "two_step_link"],
  "source_record_id": "squad-1-1",
  "source": "squad_v2",
  "difficulty_band": "intermediate",
  "output": {
    "summary": "In this study, we propose a / that reduces overload during reading",
    "recommended_style": "short-linked-sentences",
    "opener_options": ["In this study: In this study, we propose a memory safe interface."],
    "bridge_phrases": ["First,", "Next,", "Also,", "The main point is,"],
    "steps": [
      {"step": 1, "text": "In this study, we propose a.", "purpose": "opener", "risk_level": "low", "delivery_tip_ja": "...", "delivery_tip_en": "..."}
    ],
    "rescue_phrases": ["Let me say that in a shorter way."]
  }
}
```

## 学習用 prompt/target へ整形

```bash
python training/prepare_seq2seq_data.py ^
  --input training/artifacts/squad_wm_raw.jsonl ^
  --output training/artifacts/squad_wm_prepared.jsonl
```

## LoRA 学習

```bash
pip install .[training]

python training/train_seq2seq_lora.py ^
  --train-file training/artifacts/squad_wm_prepared.jsonl ^
  --eval-file training/artifacts/sample_eval_prepared.jsonl ^
  --model-name google/flan-t5-large ^
  --output-dir checkpoints/flan-t5-large-lora
```

## 補足

- `sample_train_raw.jsonl` と `sample_eval_raw.jsonl` は最小の手元検証用です
- `generated_speaking_1000.jsonl` のようなテンプレート大量生成は、補助データとしては使えますが主力教師データには向きません
- 実用優先なら `公開コーパス + 高品質の WM 特化ラベル` を主軸にしてください
