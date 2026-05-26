# Iteration 4: 新規メール受信通知 — 要件確認

方針: B+C（IMAP IDLE + フォールバックポーリング + バックグラウンド常駐）

## Question 1

IMAP IDLEの対象フォルダはどうしますか？

A) INBOXのみ（最もシンプル。RFC 2177ではIDLEは1フォルダ/接続）
B) INBOX + ユーザーが設定で選択したフォルダ（複数接続が必要）
C) Other (please describe after [Answer]: tag below)

[Answer]: a （設定画面から変更できてもいいかも。つうちたいしょうふぉるだを選択するイメージでデフォルトは受信トレイのみで、必要に応じて特定のフォルダだけとか設定できるイメージ）

## Question 2

通知の表示内容はどの程度の情報を含めますか？

A) 送信者 + 件名（現在の実装と同じ）
B) 送信者 + 件名 + 本文プレビュー（最初の数十文字）
C) Other (please describe after [Answer]: tag below)

[Answer]: a

## Question 3

複数アカウント登録時、通知対象はどうしますか？

A) 全アカウント（アカウントごとにIDLE接続を維持）
B) アカウント設定で個別にON/OFF（現在の `notifications` フラグを流用）
C) Other (please describe after [Answer]: tag below)

[Answer]: b デフォルトは 通知ONで

## Question 4

ウィンドウを閉じた後のアプリ動作はどうしますか？

A) macOS標準動作（ウィンドウ閉じてもDockに残り、クリックで再表示）
B) メニューバー常駐アイコン追加（Dockアイコンは非表示にしない）
C) メニューバー常駐 + Dockアイコン非表示（完全バックグラウンドアプリ化）
D) Other (please describe after [Answer]: tag below)

[Answer]: C

## Question 5

ネットワーク断・IDLE接続切断時のフォールバック動作は？

A) 自動再接続 + 再接続失敗時はポーリングにフォールバック（推奨）
B) 自動再接続のみ（ポーリングフォールバックなし）
C) Other (please describe after [Answer]: tag below)

[Answer]: A

## Question 6

フォールバックポーリングの間隔は？

A) 現在の設定値を流用（デフォルト5分、ユーザー変更可能）
B) IDLE失敗時は短い間隔（1分）で、通常時は長い間隔（5分）
C) Other (please describe after [Answer]: tag below)

[Answer]: A
