# Functional Design Plan — Unit 1: IdleWatcher

## Plan Steps

- [ ] ビジネスロジックモデル（状態遷移図）
- [ ] ビジネスルール（再接続ポリシー、タイムアウト等）
- [ ] ドメインエンティティ（WatcherStatus等）

## Design Questions

### Question 1

再接続のexponential backoff設定は？

A) 1s → 2s → 4s → 8s → 16s → 30s（max 30s、以降30s固定でリトライ継続）
B) 5s → 10s → 30s → 60s → 300s（max 5分、以降5分固定）
C) Other (please describe after [Answer]: tag below)

[Answer]: b

### Question 2

IDLE接続が何回連続失敗したらポーリングフォールバックに切り替えますか？

A) 3回連続失敗
B) 5回連続失敗
C) Other (please describe after [Answer]: tag below)

[Answer]: b