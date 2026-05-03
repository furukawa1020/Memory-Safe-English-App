# admin-rb

`services/admin-rb` は、教材データベースを Ruby で運用しやすくするための管理ツールです。
コア API や学習処理は `Go / Python / Rust` のままにして、Ruby は `教材追加・検証・seed 生成` に寄せています。

## できること

- YAML 教材カタログの検証
- 件数や難度帯の集計
- Go の in-memory seed ファイル生成
- PostgreSQL 用 SQL seed 生成
- 既存 SQL seed の YAML 化
- 複数 SQL seed の一括統合
- YAML から Go / SQL / stats の一括再生成

## ディレクトリ

```text
services/admin-rb
|- bin/catalog_ops
|- data/
|  `- sample_content_catalog.yml
|- lib/catalog_ops/
|  |- cli.rb
|  |- content_record.rb
|  |- catalog_loader.rb
|  |- stats.rb
|  `- renderers/
|     |- go_seed_renderer.rb
|     `- sql_seed_renderer.rb
`- test/
   `- catalog_ops_test.rb
```

## YAML 形式

```yaml
contents:
  - id: cnt_sample_001
    title: Research Reading: Main Claim
    content_type: reading
    level: intermediate
    topic: research
    language: en
    raw_text: We tested whether smaller chunks reduce overload during reading.
    summary_text: Main claim about chunk size
```

必須項目:

- `id`
- `title`
- `content_type`
- `level`
- `topic`
- `language`
- `raw_text`
- `summary_text`

## 使い方

### 検証

```bash
ruby bin/catalog_ops validate data/sample_content_catalog.yml
```

### 集計

```bash
ruby bin/catalog_ops stats data/sample_content_catalog.yml
```

### Go seed 生成

```bash
ruby bin/catalog_ops build-go data/sample_content_catalog.yml tmp/generated_seed.go --package memory --function generatedRubySeedCatalog
```

### SQL seed 生成

```bash
ruby bin/catalog_ops build-sql data/sample_content_catalog.yml tmp/generated_seed.sql
```

### YAML から Go / SQL / stats をまとめて生成

```bash
ruby bin/catalog_ops build-all data/sample_content_catalog.yml tmp/build --package memory --function generatedRubySeedCatalog
```

出力されるもの:

- `generated_seed.go`
- `generated_seed.sql`
- `catalog_stats.json`

### 既存 SQL seed を YAML へ変換

```bash
ruby bin/catalog_ops import-sql ../../infra/postgres/init/010_seed_catalog_bulk7.sql tmp/imported_catalog.yml
```

### 複数 SQL seed を 1 つの YAML に統合

```bash
ruby bin/catalog_ops import-sql-dir ../../infra/postgres/init tmp/all_catalog.yml --pattern "*seed*.sql"
```

重複した `id` が見つかった場合は、その場で失敗して止まります。

## 推奨フロー

既存 seed を Ruby 管理へ寄せたいときは、この順が扱いやすいです。

1. SQL seed 群を YAML に統合

```bash
ruby bin/catalog_ops import-sql-dir ../../infra/postgres/init tmp/all_catalog.yml --pattern "*seed*.sql"
```

2. 統合した YAML から Go / SQL / stats を再生成

```bash
ruby bin/catalog_ops build-all tmp/all_catalog.yml tmp/build --package memory --function generatedRubySeedCatalog
```

## テスト

```bash
ruby -Ilib:test test/catalog_ops_test.rb
```
