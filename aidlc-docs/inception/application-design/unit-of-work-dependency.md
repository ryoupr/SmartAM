# Unit of Work — Dependency Matrix

## Dependency Diagram

```mermaid
graph LR
    U1[Unit 1<br/>バグ修正]
    U2[Unit 2<br/>FE リファクタ]
    U3[Unit 3<br/>BE リファクタ]
    U4[Unit 4<br/>Keychain]
    U5[Unit 5<br/>パフォーマンス]
    U6[Unit 6<br/>テスト+CI]

    U3 --> U4
    U2 --> U5
    U3 --> U5
    U2 --> U6
    U3 --> U6

    style U1 fill:#f38ba8,stroke:#1e1e2e,color:#1e1e2e
    style U2 fill:#89b4fa,stroke:#1e1e2e,color:#1e1e2e
    style U3 fill:#89b4fa,stroke:#1e1e2e,color:#1e1e2e
    style U4 fill:#f9e2af,stroke:#1e1e2e,color:#1e1e2e
    style U5 fill:#f9e2af,stroke:#1e1e2e,color:#1e1e2e
    style U6 fill:#f9e2af,stroke:#1e1e2e,color:#1e1e2e
```

## Execution Order

```
Phase A (先行リリース):  Unit 1 → リリース v0.2.13
Phase B (並行):          Unit 2 + Unit 3 (同時着手可能)
Phase C (依存解決後):    Unit 4 + Unit 5 + Unit 6 (Unit 2,3完了後)
Final Release:           v0.3.0
```

## Dependency Matrix

| Unit | Depends On | Blocks | Can Parallel With |
|------|-----------|--------|-------------------|
| 1 | なし | なし | 2, 3 |
| 2 | なし | 5, 6 | 1, 3 |
| 3 | なし | 4, 5, 6 | 1, 2 |
| 4 | 3 | なし | 5, 6 |
| 5 | 2, 3 | なし | 4, 6 |
| 6 | 2, 3 | なし | 4, 5 |
