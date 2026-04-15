# Worker Service

Python 製の NLP / 音声解析ワーカーの配置先です。

初期責務:

- 英文チャンク分割
- 骨格抽出
- 音声停止点候補推定
- 発話テキスト解析

初手では、本格モデルより先にインターフェースを固定します。

推奨ディレクトリ案:

```text
services/worker
├─ app
│  ├─ chunking
│  ├─ skeleton
│  ├─ speech
│  └─ jobs
└─ tests
```
