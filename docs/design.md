# 設計（Design）

このドキュメントは、学習用のコーディングエージェント "Microde CLI"の設計文書である。

## 処理フロー

処理フローは以下のように定義される


```
user_input = 入力の受取
context = ""
Loop:
  context += ログ出力と会話履歴の取得
  context += user_input
  終了判定
  if コンテキストオーバーフロー:
    compaction(context) # コンテキスト圧縮

  context += LLM_process()

  if コンテキストオーバーフロー:
    context = compaction(context) # コンテキスト圧縮

function LLM_process():
  While ! finish:
    LLM呼び出し
    ツール呼び出し
```

この際、複雑なエラーハンドリングやDoom Loop (LLM呼び出しの無限ループ)は検出しない。

## ツール定義

実装を簡略化するため、ツール実行には必ずユーザーへのパーミッションチェックを行う。
またツールの実装全体において、最低限のセキュリティ防護策は取るが、細かな防護策は取らない。

標準ツール
| ツール名 | ファイル | 概要 |
|---------|----------|------|
| `bash`  | `src/tool/bash.rs` | シェルコマンドの実行 |
| `read`  | `src/tool/read.rs` | ファイルの読み取り |
| `edit`  | `src/tool/edit.rs` | ファイルを編集     |
| `write` | `src/tool/write.rs`| ファイル全体を書き込み。新規作成と上書きが可能|
| `grep`  | `src/tool/grep.rs` | 正規表現でファイル内容を検索 |
| `task`  | `src/tool/task.rs` | サブエージェントの起動。子セッションを作成し、独立したLLM_process()で実行 |
| `question` | `src/tool/question.rs` | ユーザーに質問をし、標準入力から回答を得る|  

## セッションとログの管理

セッションとは、一つのエージェントが持つ会話履歴を含むコンテキストのことを言います。
基本的にセッションはHashMapを用いて実装され、Microde CLIのプロセス終了を超えて永続化することは考えません。
ただし、実装自体はセッション管理を適切に抽象化することで、今後の拡張(SQLiteを使ってセッション管理を行う、永続化を目指すなど)に備えます。

ログには、ツール実行を含むMicrode CLI全体の状態や実行履歴を記録します。これはLogクレートを使って実装され、標準出力または特定のファイルに書き込まれます。
