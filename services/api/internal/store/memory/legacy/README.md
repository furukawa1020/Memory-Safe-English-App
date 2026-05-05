このディレクトリは、Ruby 生成 seed 導線へ移行する前に使っていた手書き教材 seed の退避場所です。

現在の現役導線:

- Ruby catalog / SQL import
- `generated_ruby_seed_catalog.go`
- `store.go`

この配下の Go ファイルは履歴参照用で、`services/api/internal/store/memory/store.go` からは読み込まれていません。
