export default {
  title: "Hermes へようこそ",
  subtitle:
    "あなたのマシンでローカル実行する自己進化型 AI アシスタント。プライベートで、強力で、常に学習します。",
  installIssueTitle: "インストールの問題",
  getStarted: "始める",
  retryInstall: "再インストール",
  terminalInstallHint: "ターミナルでインストールしてから戻ってきてください：",
  recheck: "インストールしました — 再チェック",
  installSizeHint: "必要なコンポーネント（約 2 GB）をインストールします",
  copyInstallCommand: "インストールコマンドをコピー",
  dividerOr: "または",
  connectRemote: "リモート Hermes に接続",
  connectRemoteTitle: "リモート Hermes に接続",
  connectRemoteSubtitle:
    "稼働中の Hermes API サーバの URL を入力してください。",
  remoteServerUrl: "サーバ URL",
  remoteApiKey: "API キー（任意）",
  remoteApiKeyPlaceholder: "Bearer トークン（API_SERVER_KEY）",
  testingConnection: "テスト中",
  connect: "接続",
  remoteHint:
    "サーバが認証なしリクエストを受け付ける（例：SSH トンネル経由で localhost）場合はキーを空欄に。",
  connectViaSsh: "SSH で接続",
  sshSubtitle:
    "SSH トンネルでリモートの Hermes に接続 — ポートの公開や API キーは不要です。",
  sshHost: "SSH ホスト",
  sshPort: "SSH ポート",
  username: "ユーザ名",
  privateKeyPath: "秘密鍵のパス",
  privateKeyPathOptional: "（任意 — デフォルトは ~/.ssh/id_rsa）",
  remoteHermesPort: "リモート Hermes ポート",
  remoteHermesPortDefault: "（デフォルト 8642）",
  testingSsh: "SSH 接続をテスト中…",
  sshHint:
    'システムの SSH を使用します。<code style="font-family:monospace;font-size:12px">ssh {user}@{host}</code> をパスワードプロンプトなしで実行できることを確認してください。',
  errorUrlRequired: "URL を入力してください。",
  errorRemoteUnreachable:
    "この URL で Hermes に接続できませんでした。URL と API キーを確認してください。\n\nサーバが認証なしリクエストを受け付ける（例：SSH トンネル経由で localhost）場合はキーを空欄にしてください。",
  errorConnectionFailed: "接続テストに失敗しました。",
  errorHostRequired: "ホストとユーザ名は必須です。",
  errorSshUnreachable:
    "SSH で接続できないか、リモートの Hermes に到達できません。以下を確認してください：\n• SSH キーが正しい（またはデフォルトの ~/.ssh/id_rsa が機能する）\n• リモートで Hermes ゲートウェイが実行中である\n• リモートポートが正しい（デフォルト 8642）",
  errorSshFailed: "SSH 接続テストに失敗しました：{error}",
} as const;
