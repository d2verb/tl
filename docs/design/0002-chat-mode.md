# Mini Design Doc: tl chat - Interactive Translation Mode

*   **Author:** d2verb
*   **Status:** Implemented
*   **Date:** 2026-01-08

## 1. Abstract

`tl chat` コマンドで、インタラクティブなチャット形式の翻訳機能を提供する。Claude Code や codex-cli、gemini-cli のようなモダンなCLI UIを採用し、スラッシュコマンドによる設定操作もサポートする。

## 2. Goals & Non-Goals

### Goals
*   インタラクティブなチャット形式での翻訳
*   モダンなCLI UI（プロンプト、色分けなど）
*   スラッシュコマンドによる設定表示・変更
*   セッション内での一時的な設定変更

### Non-Goals
*   会話履歴の永続化（v1ではセッション内のみ）
*   マルチターン翻訳（文脈を考慮した連続翻訳）
*   プラグインシステム
*   入力履歴機能（v1ではスコープ外）

## 3. Context & Problem Statement

現在の `tl` コマンドは単発の翻訳に特化しており、複数のテキストを連続して翻訳する場合は毎回コマンドを実行する必要がある。インタラクティブモードを追加することで、連続翻訳のワークフローを改善し、設定の確認・変更も容易にする。

## 4. Proposed Design

### 4.1 CLI Interface

```bash
# チャットモード起動
tl chat

# オプション付きで起動
tl chat -t ja -e http://localhost:11434 -m llama3.2
```

### 4.2 UI Design

```
$ tl chat -t ja

tl v0.1.0 - Interactive Translation Mode
Type text to translate, or use /commands. Press Ctrl+C to exit.

> Hello, world!
こんにちは、世界！

> How are you?
お元気ですか？

> /config
Current configuration:
  to       = ja
  endpoint = http://localhost:11434
  model    = llama3.2

> /set to en
Set 'to' to 'en'

> こんにちは
Hello

> /help
Available commands:
  /config             Show current configuration
  /set <key> <value>  Set configuration (session only)
  /clear              Clear screen
  /help               Show this help
  /quit               Exit chat mode

> /quit
Goodbye!
```

### 4.3 Slash Commands

| Command | Description |
|:--------|:------------|
| `/config` | 現在の設定を表示 |
| `/set <key> <value>` | 設定を変更（セッション内のみ有効） |
| `/clear` | 画面をクリア |
| `/help` | ヘルプを表示 |
| `/quit` または `/exit` | チャットモードを終了 |

**設定可能なキー:**
*   `to` - 翻訳先言語
*   `endpoint` - APIエンドポイント
*   `model` - モデル名

### 4.4 Session Configuration

```
起動時の設定読み込み優先順位:
1. CLI引数 (最優先)
2. config.toml
3. デフォルト値 (なし → エラー)

セッション中:
- /set で変更した値はセッション内でのみ有効
- config.toml は変更しない
- 終了時に変更は破棄
```

### 4.5 UI Components

```
┌─────────────────────────────────────────────────────────┐
│  tl v0.1.0 - Interactive Translation Mode               │
│  Type text to translate, or use /commands.              │
│  Press Ctrl+C to exit.                                  │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  > [user input]                                         │
│  [translated output]                                    │
│                                                         │
│  > [user input]                                         │
│  ⠋ Translating...                                       │
│  [streaming output...]                                  │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

**UI要素:**
*   ヘッダー: バージョン、モード、基本操作説明
*   プロンプト: `> ` で入力待ち
*   出力: 翻訳結果（ストリーミング）
*   スピナー: 翻訳中の表示
*   色分け:
    *   プロンプト `>` : 緑
    *   コマンド `/...` : 青
    *   エラー: 赤
    *   翻訳結果: デフォルト

### 4.6 Module Structure

```
src/
├── cli/
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── translate.rs
│   │   ├── configure.rs
│   │   └── chat.rs          # 新規: チャットモード
│   └── ...
├── chat/                     # 新規: チャット機能
│   ├── mod.rs
│   ├── session.rs            # セッション管理
│   ├── command.rs            # スラッシュコマンド処理
│   └── ui.rs                 # UI表示
└── ...
```

### 4.7 Dependencies

追加で必要なクレート:

```toml
[dependencies]
rustyline = "15"              # 対話的プロンプト（dialoguer を置き換え）
```

**注:** 既存の `dialoguer` を `rustyline` に統一する。`tl configure` コマンドも `rustyline` を使用するようにリファクタリングする。`rustyline` は日本語などのマルチバイト文字のカーソル位置を正しく処理できる。

### 4.8 Data Structures

```rust
/// チャットセッションの設定（セッション内で変更可能）
pub struct SessionConfig {
    pub to: String,
    pub endpoint: String,
    pub model: String,
}

/// スラッシュコマンド
pub enum SlashCommand {
    Config,
    Set { key: String, value: String },
    Clear,
    Help,
    Quit,
}

/// 入力の種類
pub enum Input {
    Text(String),           // 翻訳対象テキスト
    Command(SlashCommand),  // スラッシュコマンド
    Empty,                  // 空入力
}
```

### 4.9 Command Parsing

```rust
fn parse_input(input: &str) -> Input {
    let input = input.trim();

    if input.is_empty() {
        return Input::Empty;
    }

    if let Some(cmd) = input.strip_prefix('/') {
        parse_slash_command(cmd)
    } else {
        Input::Text(input.to_string())
    }
}

fn parse_slash_command(cmd: &str) -> Input {
    let parts: Vec<&str> = cmd.split_whitespace().collect();

    match parts.as_slice() {
        ["config"] => Input::Command(SlashCommand::Config),
        ["set", key, value] => Input::Command(SlashCommand::Set {
            key: key.to_string(),
            value: value.to_string(),
        }),
        ["clear"] => Input::Command(SlashCommand::Clear),
        ["help"] => Input::Command(SlashCommand::Help),
        ["quit"] | ["exit"] | ["q"] => Input::Command(SlashCommand::Quit),
        _ => {
            // Unknown command
            Input::Text(format!("Unknown command: /{}", parts.join(" ")))
        }
    }
}
```

### 4.10 Main Loop

```rust
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

async fn run_chat(initial_config: SessionConfig) -> Result<()> {
    print_header();

    let mut config = initial_config;
    let mut rl = DefaultEditor::new()?;

    loop {
        let input = rl.readline("> ");

        match input {
            Ok(line) => {
                match parse_input(&line) {
                    Input::Empty => {}
                    Input::Command(cmd) => {
                        if !handle_command(&mut config, cmd) {
                            break;  // /quit
                        }
                    }
                    Input::Text(text) => {
                        translate_and_print(&config, &text).await?;
                    }
                }
            }
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => break,  // Ctrl+C or Ctrl+D
            Err(e) => return Err(e.into()),
        }
    }

    println!("Goodbye!");
    Ok(())
}
```

## 5. Implementation Plan

1.  **Phase 1: 依存関係の整理**
    *   `dialoguer` を `rustyline` に置き換え
    *   `tl configure` コマンドを `rustyline` でリファクタリング

2.  **Phase 2: 基本構造**
    *   `tl chat` サブコマンドの追加
    *   `SessionConfig` の実装
    *   基本的なREPLループ

3.  **Phase 3: スラッシュコマンド**
    *   コマンドパーサーの実装
    *   `/config`, `/set`, `/help`, `/quit` の実装
    *   `/clear` の実装

4.  **Phase 4: UI改善**
    *   色分け表示
    *   ストリーミング翻訳との統合

5.  **Phase 5: 品質向上**
    *   エラーハンドリング
    *   テストの追加

## 6. Risks & Mitigations

| Risk | Impact | Mitigation Strategy |
| :--- | :--- | :--- |
| rustyline のクロスプラットフォーム互換性 | Low | rustyline は広く使われており安定している |
| ストリーミング出力とプロンプトの競合 | Low | 翻訳中はプロンプトを非表示にする |
| 長いテキスト入力の扱い | Low | 複数行入力は将来の拡張として検討 |

## 7. Testing & Verification

### Unit Tests
*   スラッシュコマンドのパース
*   SessionConfig の初期化・更新
*   入力の分類（テキスト vs コマンド）

### Integration Tests
*   チャットセッションの起動・終了
*   設定変更の反映
*   翻訳の実行

### Manual Testing
*   各ターミナルエミュレータでのUI確認
*   Ctrl+C の動作確認

## 8. Alternatives Considered

### 入力方式

| 選択肢 | メリット | デメリット | 決定 |
| :--- | :--- | :--- | :--- |
| rustyline | 履歴、補完、行編集、Unicode対応 | APIがやや複雑 | **採用** |
| inquire | シンプルなAPI、モダンなUI | 日本語などのマルチバイト文字でカーソル位置がずれる | 不採用 |
| 標準入力のみ | シンプル | 編集なし、UXが悪い | 不採用 |
| ratatui (TUI) | リッチなUI | 複雑、オーバーエンジニアリング | 不採用 |

### コマンドプレフィックス

| 選択肢 | メリット | デメリット | 決定 |
| :--- | :--- | :--- | :--- |
| `/command` | 直感的、他ツールと同様 | 翻訳テキストが `/` で始まる場合の対応 | **採用** |
| `:command` | Vim風 | あまり一般的でない | 不採用 |
| `!command` | シェル風 | `!` は翻訳テキストに使われやすい | 不採用 |

---

## Appendix: Future Enhancements (Out of Scope for v1)

*   複数行入力のサポート（ヒアドキュメント風）
*   入力履歴の永続化
*   コマンド補完
*   カスタムプロンプト
*   マルチターン翻訳（文脈保持）
