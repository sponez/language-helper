# Main Screen - Create New User Modal
create-new-user-title = 新しいユーザーを作成
username-placeholder = ユーザー名
choose-language-placeholder = 言語を選択
ok-button = OK
cancel-button = キャンセル

# Main Screen - User Selection
user-list-select-placeholder = ユーザーを選択

# Main Screen - Validation Errors
error-username-too-short = ユーザー名は5文字以上である必要があります
error-username-too-long = ユーザー名は50文字を超えることはできません
error-language-not-selected = 言語を選択してください

# Main Screen - API Errors
error-create-user = ユーザーの作成に失敗しました
error-update-theme = テーマの更新に失敗しました
error-update-language = 言語の更新に失敗しました
error-load-user-settings = ユーザー設定の読み込みに失敗しました

# User Screen - Navigation
user-back-button = 戻る
user-profiles-button = プロフィール
user-settings-button = 設定

# User Screen - Title
user-account-title = アカウント: {$username} | 言語: {$language}

# User Settings Screen
user-settings-language-label = 言語:
user-settings-theme-label = テーマ:
user-settings-delete-button = ユーザーを削除
user-settings-back-button = 戻る
user-settings-delete-warning = このユーザーを削除してもよろしいですか？すべてのプロフィールとデータが削除されます。
user-settings-delete-yes = はい、削除します
user-settings-delete-no = キャンセル
user-settings-api-error-theme = テーマの更新に失敗しました
user-settings-api-error-delete = ユーザーの削除に失敗しました

# Error Modal
error-modal-close-button = 閉じる

# Loading State
loading = 読み込み中...

# Profile List Screen
profile-list-title = プロフィールを選択
profile-list-back-button = 戻る

# Create New Profile Modal
create-new-profile-title = 新しいプロフィールを作成
profile-name-placeholder = プロフィール名
profile-language-placeholder = 対象言語を選択
profile-ok-button = 作成
profile-cancel-button = キャンセル

# Profile List - Validation Errors
error-profile-name-too-short = プロフィール名は5文字以上である必要があります
error-profile-name-too-long = プロフィール名は50文字を超えることはできません
error-profile-language-not-selected = 対象言語を選択してください

# Profile List - API Errors
error-create-profile = プロフィールの作成に失敗しました
error-load-profiles = プロフィールの読み込みに失敗しました

# Profile Screen
profile-title = プロフィール: {$profile} | 学習中: {$language}
profile-back-button = 戻る
profile-cards-button = カード
profile-explain-ai-button = AI解説
profile-settings-button = 設定

# Profile Screen - API Errors
error-load-card-settings = カード設定の読み込みに失敗しました

# Profile Settings Screen
profile-settings-back-button = 戻る
profile-settings-card-settings-button = カード設定
profile-settings-assistant-settings-button = AIアシスタント設定
profile-settings-delete-profile = プロフィールを削除
profile-settings-delete-warning = このプロフィールを削除してもよろしいですか？すべてのカードと進捗が完全に削除されます。
profile-settings-delete-yes = はい、削除します
profile-settings-delete-no = キャンセル
profile-settings-api-error-delete = プロフィールの削除に失敗しました

# Card Settings Screen
card-settings-title = カード設定
card-settings-back-button = 戻る
card-settings-cards-per-set = セットあたりのカード数:
card-settings-test-method = テスト方法:
card-settings-test-method-manual = 手動入力
card-settings-test-method-self = 自己評価
card-settings-streak-length = ストリーク長:
card-settings-save = 設定を保存
card-settings-saved = 設定が正常に保存されました
error-cards-per-set-range = セットあたりのカード数は1から100の間である必要があります
error-streak-length-range = ストリーク長は1から50の間である必要があります
error-invalid-number = 有効な数値を入力してください
error-save-card-settings = カード設定の保存に失敗しました

# Assistant Settings Screen
assistant-settings-title = AIアシスタント設定
assistant-settings-back-button = 戻る
assistant-settings-model-label = モデル:
assistant-settings-tiny = 極小
assistant-settings-light = 軽量
assistant-settings-weak = 弱
assistant-settings-medium = 中
assistant-settings-strong = 強
assistant-settings-api = API

# API Configuration
assistant-settings-api-endpoint = APIエンドポイント:
assistant-settings-api-key = APIキー:
assistant-settings-api-model = モデル名:

# System Requirements
assistant-settings-requirements-title = システム要件
assistant-settings-incompatible = お使いのシステムはこのモデルの要件を満たしていません
assistant-settings-no-data = システム要件を確認できません
assistant-settings-ollama-not-installed = Ollamaがインストールされていません。インストールするには次のサイトにアクセスしてください

# Actions
assistant-settings-start-assistant = アシスタントを起動
assistant-settings-stop-assistant = アシスタントを停止
assistant-settings-change-assistant = アシスタントを変更
assistant-settings-save-api = API設定を保存
assistant-settings-download = ダウンロード
assistant-settings-cancel = キャンセル
assistant-settings-close = 閉じる

# Launch Modal
assistant-settings-launching = アシスタントを起動しています。しばらくお待ちください...
assistant-settings-checking-server = Ollamaサーバーの状態を確認中...
assistant-settings-starting-server = Ollamaサーバーを起動中...
assistant-settings-checking-models = 利用可能なモデルを確認中...
assistant-settings-pulling-model = モデルをダウンロード中...
assistant-settings-launching-model = モデルを起動中...
assistant-settings-launch-success = モデルが正常に起動されました！
assistant-settings-launch-error = アシスタントの起動に失敗しました

# API Errors
error-load-assistant-settings = アシスタント設定の読み込みに失敗しました
error-save-assistant-settings = アシスタント設定の保存に失敗しました
error-clear-assistant-settings = アシスタント設定のクリアに失敗しました

# Explain AI Screen
explain-ai-title = AI解説
explain-ai-back = 戻る
explain-ai-input-label = 説明するフレーズを入力:
explain-ai-send = 送信
explain-ai-response-label = AI解説:
explain-ai-placeholder = ここに解説が表示されます...
explain-ai-loading = 応答を生成中...

# Cards Menu Screen
cards-menu-title = カードメニュー
cards-menu-back = 戻る
cards-menu-manage = カード管理
cards-menu-learn = 学習
cards-menu-test = テスト
cards-menu-repeat = 復習

# Manage Cards Screen
manage-cards-title = カード管理
manage-cards-back = 戻る
manage-cards-unlearned-tab = 未学習
manage-cards-learned-tab = 学習済み
manage-cards-no-unlearned = まだ未学習のカードはありません
manage-cards-no-learned = まだ学習済みのカードはありません
manage-cards-edit = 編集
manage-cards-delete = 削除
manage-cards-add-new = 新しいカードを追加

# Add Card Screen
add-card-title = カードを追加
add-card-fill-ai = AIで入力
add-card-ai-filling = AIがカードを入力しています...
add-card-type-label = カードタイプ:
add-card-type-straight = 通常
add-card-type-reverse = 逆
add-card-word-label = 単語:
add-card-word-placeholder = 単語名を入力
add-card-readings-label = 読み方（オプション）:
add-card-reading-placeholder = 読み方を入力
add-card-add-reading = 読み方を追加
add-card-meanings-label = 意味:
add-card-definition-label = 定義:
add-card-definition-placeholder = 定義を入力
add-card-translated-def-label = 翻訳された定義（オプション）:
add-card-translated-def-placeholder = 翻訳された定義を入力
add-card-translations-label = 翻訳:
add-card-translation-placeholder = 翻訳を入力
add-card-add-translation = 翻訳を追加
add-card-remove-meaning = 意味を削除
add-card-add-meaning = 意味を追加
add-card-save = 保存
add-card-cancel = キャンセル
add-card-inverse-modal-title = 逆カードを作成しますか？
add-card-inverse-manually = 手動で
add-card-inverse-with-assistant = AIアシスタントで
add-card-inverse-no = いいえ

# Inverse Cards Review Screen
inverse-cards-review-title = 逆カードを確認
inverse-cards-back = 戻る
inverse-cards-no-pending = 保留中の逆カードはありません
inverse-cards-edit = 編集
inverse-cards-delete = 削除
inverse-cards-save-all = すべて保存
inverse-cards-skip-all = すべてスキップ
inverse-cards-saving = カードを保存中...

# Learn Router - Start Screen
learn-title = 学習モード
learn-start-instruction = 開始カード番号を入力:
learn-card-number-placeholder = カード番号
learn-start-button = 開始
learn-back = 戻る

# Learn Router - Loading
learn-loading = セッションを読み込み中...

# Learn Router - Study Phase
learn-foreign-word-label = 単語:
learn-readings-label = 読み方:
learn-meanings-label = 意味:
learn-next-card = 次のカード
learn-start-test = テスト開始
learn-no-cards = 利用可能なカードがありません

# Learn Router - Test Phase
learn-answer-label = あなたの回答:
learn-remaining-answers = 残りの回答
learn-answer-placeholder = 回答を入力
learn-submit-answer = 送信
learn-correct = 正解
learn-incorrect = 不正解
learn-continue = 続ける

# Learn Router - Self-Review Mode
learn-show-answer = 答えを表示
learn-answer-correct = 正解
learn-answer-incorrect = 不正解

# Learn Router - Results
learn-test-passed = テスト合格！
learn-test-failed = テスト不合格
learn-passed-message = おめでとうございます！このカードセットをマスターしました。
learn-failed-message = 練習を続けましょう！このセットを再挑戦できます。
learn-next-set = 次のセット
learn-retry-set = セットを再挑戦

# Test Router - Loading
test-loading = テストを読み込み中...
test-back = 戻る
test-no-cards = テスト可能なカードがありません

# Test Router - Results
test-test-passed = テスト合格！
test-test-failed = テスト不合格
test-passed-message = よくできました！すべての未学習単語のテストが成功しました。
test-failed-message = いくつかの回答が間違っていました。練習を続けましょう！
test-retry-test = テストを再挑戦

# Repeat Router - Loading
repeat-loading = 復習セッションを読み込み中...
repeat-back = 戻る
repeat-no-cards = 復習可能な学習済みカードがありません

# Repeat Router - Results
repeat-test-passed = 復習合格！
repeat-test-failed = 復習不合格
repeat-passed-message = 素晴らしい！学習済みの単語をすべて覚えていました。
repeat-failed-message = いくつかの単語はもっと練習が必要です。未学習に戻されました。
repeat-retry-repeat = 復習を再挑戦
