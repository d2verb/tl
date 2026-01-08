# Mini Design Doc: tl - AI Translation CLI Tool

*   **Author:** d2verb
*   **Status:** Draft
*   **Date:** 2026-01-08

## 1. Abstract

`tl` は、OpenAI互換APIを利用してファイルや標準入力のテキストを翻訳するCLIツールである。ストリーミング出力とキャッシュ機能により、ユーザー体験を損なわずに効率的な翻訳を提供する。

## 2. Goals & Non-Goals

### Goals
*   ファイルまたは標準入力からテキストを読み取り、指定した言語に翻訳する
*   OpenAI互換API（ローカルLLMサーバー等）を利用した翻訳
*   ストリーミング出力による低レイテンシなUX
*   翻訳結果のキャッシュによる再翻訳の高速化
*   `tl configure` による対話的な設定管理

### Non-Goals
*   翻訳APIの自前実装（外部APIに依存）
*   複数ファイルの一括翻訳（v1では単一ファイル/標準入力のみ）
*   翻訳履歴の管理UI
*   オフライン翻訳

## 3. Context & Problem Statement

テキストファイルを手軽に翻訳したい場面は多いが、既存のCLIツールは以下の課題がある:
*   商用API専用で、ローカルLLMに対応していない
*   ストリーミング出力に対応しておらず、長文の翻訳時に長時間ブロックされる
*   キャッシュ機能がなく、同じ文章を何度も翻訳するとコストがかかる

本ツールは、OpenAI互換APIをサポートすることでローカルLLMを含む様々なバックエンドに対応し、ストリーミング出力とキャッシュで快適な翻訳体験を提供する。

## 4. Proposed Design

### 4.1 System Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                           tl CLI                                │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │   Input     │  │   Config    │  │      Translation        │  │
│  │   Handler   │  │   Manager   │  │        Engine           │  │
│  │             │  │             │  │                         │  │
│  │ - File      │  │ - TOML      │  │ - OpenAI Compatible API │  │
│  │ - Stdin     │  │ - CLI Args  │  │ - Streaming Response    │  │
│  └──────┬──────┘  └──────┬──────┘  └───────────┬─────────────┘  │
│         │                │                     │                │
│         └────────────────┼─────────────────────┘                │
│                          │                                      │
│                   ┌──────▼──────┐                               │
│                   │    Cache    │                               │
│                   │   Manager   │                               │
│                   │             │                               │
│                   │ - SHA256    │                               │
│                   │ - SQLite    │                               │
│                   └─────────────┘                               │
└─────────────────────────────────────────────────────────────────┘
```

### 4.2 CLI Interface

```bash
# 基本的な使い方
tl <file>                    # ファイルを翻訳（デフォルト言語へ）
cat <file> | tl              # 標準入力から翻訳
tl -t <lang> <file>          # 翻訳先言語を指定
tl --to <lang> <file>        # 同上（長いオプション）

# 設定管理
tl configure                 # 対話的に設定を編集
tl configure show            # 現在の設定を表示

# その他オプション
tl --help                    # ヘルプ表示
tl --version                 # バージョン表示
tl --list-languages          # サポートする言語コード一覧を表示
tl --no-cache <file>         # キャッシュを使用しない
tl --endpoint <url> <file>   # APIエンドポイントを一時的に指定
tl --model <name> <file>     # モデルを一時的に指定
```

### 4.3 Configuration

**ファイルパス:** `~/.config/tl/config.toml`

```toml
[tl]
to = "ja"                              # 翻訳先言語（必須）
endpoint = "http://localhost:11434"    # APIエンドポイント（必須）
model = "gpt-oss:20b"                  # モデル名（必須）
```

**設定の優先順位:**
1. CLIオプション（最優先）
2. 設定ファイル
3. エラー（必須項目が未設定の場合）

**必須設定が不足している場合のエラー:**
```
Error: Missing required configuration: 'endpoint'

Please provide it via:
  - CLI option: tl --endpoint <url> <file>
  - Config file: Run 'tl configure' to set up configuration
```

### 4.4 Caching Strategy

**言語コード:**

言語コードはISO 639-1形式を強制する（`ja`, `en`, `zh`, `ko`, `fr`, `de`, `es` 等）。

*   正規化ロジックが不要になりシンプル
*   キャッシュキーの一貫性が保証される
*   無効なコードはバリデーションエラーとして即座に拒否

```
Error: Invalid language code: 'Japanese'

Valid language codes (ISO 639-1): ja, en, zh, ko, fr, de, es, ...
Run 'tl --list-languages' to see all supported codes.
```

**キャッシュキー生成:**

単純な文字列連結は衝突の可能性があるため、JSON形式でシリアライズしてからハッシュ化する。

```rust
let cache_input = serde_json::json!({
    "source_text": source_text,
    "target_language": target_language,
    "model": model,
    "endpoint": endpoint,
    "prompt_hash": prompt_hash
});
let cache_key = SHA256(cache_input.to_string());
```

*   JSON化により、フィールド間の境界が明確になり衝突を防止
*   `endpoint`: 同じモデル名でも異なるサーバー（本番 vs ローカル等）での翻訳結果を区別
*   `prompt_hash`: システムプロンプトテンプレートのSHA256ハッシュ。プロンプト変更時に自動でキャッシュ無効化

**キャッシュストレージ:** `~/.cache/tl/translations.db` (SQLite)

**スキーマ:**
```sql
CREATE TABLE translations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cache_key TEXT UNIQUE NOT NULL,
    source_text TEXT NOT NULL,
    translated_text TEXT NOT NULL,
    target_language TEXT NOT NULL,
    model TEXT NOT NULL,
    endpoint TEXT NOT NULL,
    prompt_hash TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    accessed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_cache_key ON translations(cache_key);
```

**ストリーミング中のCtrl+C対応:**
*   ストリーミング中はメモリ上にバッファリング
*   完了時のみDBに書き込み
*   Ctrl+Cで中断された場合はキャッシュに保存しない（不完全な翻訳を防ぐ）

### 4.5 Input & Output Processing

**入力処理:**

入力は全て読み込んでから翻訳を開始する（ストリーミング入力は行わない）。

```
┌─────────────────┐          ┌─────────────────┐
│  File           │──read───▶│                 │
└─────────────────┘          │  Complete Text  │───▶ Translation
┌─────────────────┐          │  (in memory)    │
│  Stdin (EOF待ち) │──read───▶│                 │
└─────────────────┘          └─────────────────┘
```

*   ファイル入力: ファイル全体を読み込む
*   標準入力: EOFまで全て読み込む（`cat file | tl`, `curl ... | tl` 等との連携用）
*   入力が揃ってから翻訳を開始することで、キャッシュキーの計算が確定する

**最大入力サイズ:**

OOMを防ぐため、入力サイズに上限を設ける。

*   最大入力サイズ: **1MB**（デフォルト）
*   超過時はエラーで終了

```
Error: Input size (2.5 MB) exceeds maximum allowed size (1 MB).

Consider splitting the file into smaller parts.
```

*   1MBあれば一般的なドキュメント翻訳には十分
*   将来的に `--max-size` オプションで上限を変更可能にすることも検討

**出力処理（ストリーミング）:**

APIからのレスポンスはストリーミングで処理し、翻訳結果をリアルタイム表示する。

```
┌──────────┐     ┌──────────────┐     ┌──────────────┐
│  Input   │────▶│   API Call   │────▶│   Stdout     │
│  Text    │     │  (Streaming) │     │  (Real-time) │
└──────────┘     └──────┬───────┘     └──────────────┘
                        │
                        ▼
                 ┌──────────────┐
                 │   Buffer     │
                 │  (In-Memory) │
                 └──────┬───────┘
                        │ (on complete)
                        ▼
                 ┌──────────────┐
                 │    Cache     │
                 │   (SQLite)   │
                 └──────────────┘
```

*   長文翻訳でもユーザーを待たせない
*   出力中はメモリ上にバッファリングし、完了時にキャッシュへ書き込み

**UI表示:**
*   翻訳開始時: スピナー表示（stderr）
*   ストリーミング中: リアルタイムでテキスト出力（stdout）
*   完了時: スピナー停止

```
⠋ Translating...
[翻訳されたテキストがストリーミングで表示]
```

### 4.6 Module Structure

```
src/
├── main.rs              # エントリーポイント、CLIパーサー
├── lib.rs               # ライブラリルート
├── cli/
│   ├── mod.rs
│   ├── args.rs          # CLI引数定義（clap）
│   └── commands/
│       ├── mod.rs
│       ├── translate.rs # 翻訳コマンド
│       └── configure.rs # 設定コマンド
├── config/
│   ├── mod.rs
│   └── manager.rs       # 設定ファイルの読み書き
├── translation/
│   ├── mod.rs
│   ├── client.rs        # OpenAI互換APIクライアント
│   └── prompt.rs        # 翻訳プロンプト生成
├── cache/
│   ├── mod.rs
│   └── sqlite.rs        # SQLiteキャッシュ実装
├── input/
│   ├── mod.rs
│   └── reader.rs        # ファイル/標準入力の読み取り
└── ui/
    ├── mod.rs
    └── spinner.rs       # スピナー表示
```

### 4.7 Dependencies (Cargo.toml)

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }       # CLI引数パーサー
tokio = { version = "1", features = ["full"] }        # 非同期ランタイム
reqwest = { version = "0.12", features = ["stream"] } # HTTPクライアント
serde = { version = "1", features = ["derive"] }      # シリアライズ
serde_json = "1"                                      # JSON処理
toml = "0.8"                                          # TOML設定ファイル
rusqlite = { version = "0.32", features = ["bundled"] } # SQLiteキャッシュ
sha2 = "0.10"                                         # ハッシュ計算
hex = "0.4"                                           # 16進数エンコード
indicatif = "0.17"                                    # スピナー/プログレス表示
dialoguer = "0.11"                                    # 対話的入力
dirs = "5"                                            # プラットフォーム固有ディレクトリ
thiserror = "2"                                       # エラー定義
anyhow = "1"                                          # エラーハンドリング
futures = "0.3"                                       # ストリーム処理

[dev-dependencies]
tempfile = "3"                                        # テスト用一時ファイル
mockito = "1"                                         # HTTPモック
assert_cmd = "2"                                      # CLIテスト
predicates = "3"                                      # アサーション
```

### 4.8 API Request Format

OpenAI互換APIへのリクエスト:

```json
POST {endpoint}/v1/chat/completions
Content-Type: application/json

{
  "model": "{model}",
  "messages": [
    {
      "role": "system",
      "content": "You are a translator. Translate the following text to {target_language}. Output only the translated text without any explanations."
    },
    {
      "role": "user",
      "content": "{source_text}"
    }
  ],
  "stream": true
}
```

*   `{model}`: 設定またはCLIオプションで指定されたモデル名
*   `{target_language}`: 設定またはCLIオプション (`--to`) で指定された翻訳先言語
*   `{source_text}`: 翻訳対象のテキスト

### 4.9 Configure Command Flow

```
$ tl configure

tl Configuration
────────────────
Target language (to) [ja]: en
API endpoint [http://localhost:11434]:
Model name [gpt-oss:20b]: llama3.2

Configuration saved to ~/.config/tl/config.toml
```

*   既存の設定値がある場合はデフォルト値として `[...]` 内に表示
*   Enterのみで既存値を維持
*   新しい値を入力すると上書き

## 5. Implementation Plan

1.  **Phase 1: 基盤構築**
    *   プロジェクト構造のセットアップ
    *   CLI引数パーサーの実装（clap）
    *   設定ファイルの読み書き実装
    *   `tl configure` / `tl configure show` の実装

2.  **Phase 2: 翻訳機能**
    *   入力ハンドラー（ファイル/標準入力）の実装
    *   OpenAI互換APIクライアントの実装
    *   ストリーミングレスポンス処理
    *   スピナーUI実装

3.  **Phase 3: キャッシュ機能**
    *   SQLiteキャッシュの実装
    *   キャッシュヒット/ミス処理
    *   `--no-cache` オプション対応

4.  **Phase 4: 品質向上**
    *   エラーハンドリングの改善
    *   ユニットテスト・統合テストの追加
    *   ドキュメント整備

## 6. Risks & Mitigations

| Risk | Impact | Mitigation Strategy |
| :--- | :--- | :--- |
| APIレスポンス形式の差異 | High | OpenAI互換の標準形式に準拠。主要なLLMサーバー（Ollama, vLLM等）でテスト |
| 大きなファイルでのメモリ使用量 | Medium | チャンク分割翻訳は将来の拡張として検討。v1では警告表示のみ |
| ストリーミング中断時のデータ不整合 | Low | 完了時のみキャッシュに書き込む設計で対応済み |
| SQLiteの並行アクセス | Low | 単一プロセスでの使用を想定。WALモードで軽減 |

## 7. Testing & Verification

### Unit Tests
*   設定ファイルのパース/シリアライズ
*   キャッシュキー生成ロジック
*   CLI引数のパース
*   入力ソース判定（ファイル vs 標準入力）

### Integration Tests
*   `tl configure` の対話的フロー（モック入力）
*   翻訳APIとの通信（モックサーバー）
*   キャッシュヒット時の動作
*   エラーケース（ファイル不存在、API接続失敗等）

### Success Metrics
*   テストカバレッジ 85%以上
*   ストリーミング開始までの初期レイテンシ < 500ms（ネットワーク除く）
*   キャッシュヒット時のレスポンス < 50ms

## 8. Alternatives Considered

### キャッシュストレージ

| 選択肢 | メリット | デメリット | 決定 |
| :--- | :--- | :--- | :--- |
| SQLite | 堅牢、クエリ可能、単一ファイル | 依存追加 | **採用** |
| JSON/TOML ファイル | シンプル | 大量データで遅い、並行アクセス問題 | 不採用 |
| sled (embedded DB) | Rust native | メンテナンス状況が不透明 | 不採用 |

### 非同期ランタイム

| 選択肢 | メリット | デメリット | 決定 |
| :--- | :--- | :--- | :--- |
| tokio | 成熟、エコシステム充実 | バイナリサイズ増加 | **採用** |
| async-std | 軽量 | エコシステムが小さい | 不採用 |
| 同期処理 | シンプル | ストリーミング実装が複雑 | 不採用 |

---

## Appendix: Future Enhancements (Out of Scope for v1)

*   複数ファイルの一括翻訳
*   翻訳元言語の自動検出
*   チャンク分割による大規模ファイル対応
*   キャッシュの有効期限設定
*   翻訳品質のフィードバック機能
