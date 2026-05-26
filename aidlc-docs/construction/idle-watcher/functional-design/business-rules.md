# Business Rules — Unit 1: IdleWatcher

## BR-1: IDLE接続管理
- アカウントの `notifications: true` かつ `notificationFolders` が空でない場合のみIDLE接続を開始
- 1アカウント×1フォルダにつき1つのIDLE接続を維持
- IDLE接続はアプリ起動時（setup内）に自動開始

## BR-2: RFC 2177 IDLE準拠
- IDLEコマンド発行後、最大29分でDONE→再IDLE（サーバータイムアウト防止）
- サーバーからの応答（EXISTS等）を受信したらIDLE終了→新着取得→IDLE再発行

## BR-3: 再接続ポリシー（Exponential Backoff）
- 接続失敗時のリトライ間隔: 5s → 10s → 30s → 60s → 300s
- 最大リトライ回数: 5回（連続失敗でフォールバック切替）
- 成功時にリトライカウンタをリセット

## BR-4: フォールバックポーリング
- 5回連続IDLE接続失敗でポーリングモードに切替
- ポーリング間隔: アカウント設定の `syncInterval`（デフォルト5分）
- 10回ポーリングごとにIDLE復帰を試行
- IDLE復帰成功時にポーリングを停止しIDLEモードに戻る

## BR-5: 通知発火条件
- 新着メールのUIDが前回取得時の最大UIDより大きい
- アカウントの `notifications: true`
- 通知一時停止（pause）が無効
- 対象フォルダが `notificationFolders` に含まれる

## BR-6: OAuthトークン管理
- IDLE接続前にトークン有効期限を確認
- 期限切れ60秒前にリフレッシュ実行
- リフレッシュ失敗時は接続失敗として再接続フローに入る

## BR-7: 設定変更時の動作
- `restart_idle_watcher` コマンドで全接続を停止→再起動
- アカウント追加/削除/通知設定変更時にFEから呼び出し

## BR-8: 通知一時停止
- `set_notification_pause(true)` で通知送信を抑制（IDLE接続は維持）
- 新着検知→イベント発火は継続（FE側のUI更新は止めない）
- macOS通知とTrayバッジ更新のみ抑制
