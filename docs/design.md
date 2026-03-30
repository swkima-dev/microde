# 設計（Design）

このドキュメントは、学習用のコーディングエージェント "Microde CLI"の設計文書である。

## 処理フロー

処理フローは以下のように定義される


```
メッセージ履歴の初期化
~~
~~
user_input = ユーザーメッセージの受信

loop:
  メッセージ履歴の取得
  
  if 終了判定:
    break
  if コンテキストオーバーフロー:
    コンテキスト圧縮

  main_agent()

fn main_agent():
  システムプロンプトの組み立て
  ツールセットの解決
  loop:
    response = LLM呼び出し
    if response == 終了判定:
      break
    match response{
      ツール呼び出し
      その他
    }
```

この際、複雑なエラーハンドリングやDoom Loop (LLM呼び出しの無限ループ)は検出しない。

## エージェント

Rigの `Agent` 構造体は `chat()`と `completion()`という2つのメソッドを実装していて、
これら2つの実装はかなり意味が異なります

### `chat()`

`chat()`は、プロンプトと `Vec<Message>`を受け取ってLLMを呼び出しますが、呼び出しの過程で
行われたtool useや推論のトークンは最終的に捨てられます。
一方で、tool useを受けたツール実行とLLM呼び出しの再処理がラッパーされているため、実装が楽です。

┌─────────────────────────────────────────────────────┐
│  agent.chat("計算して", external_history) の内部     │
│                                                     │
│  internal_history = external_history.clone()        │
│                                                     │
│  loop {                                             │
│    resp = model.completion(prompt, internal_history) │
│                                                     │
│    match resp {                                     │
│      Text(t) => return Ok(t)  ← String だけ返す    │
│                                                     │
│      ToolCall(tc) => {                              │
│        result = toolset.call(tc)                    │
│        internal_history.push(AssistantContent::ToolCall) │
│        internal_history.push(UserContent::ToolResult)    │
│        // ↑ 内部にだけ積む。external_history は変わらない │
│        continue                                     │
│      }                                              │
│    }                                                │
│  }                                                  │
└─────────────────────────────────────────────────────┘

### `completion()`

これはLLMの呼び出しをAgentとして抽象化（ラップした） `chat()`とは異なり、
Agentが実装した `Completion` トレイトから `CompletionRequestBuilder`構造体を作成するメソッドです。

これによってAgentに設定済みのpreamble, temperature, toolsがプリセット済みの状態で、
さらに上書き・追加して `send()`メソッドを呼ぶことでLLM呼び出しが行われます

### 最終的なエージェント設計

`chat()`と`completion()`の違いから、以下が言えます

- Coding Agent のメインとなるエージェントは `completion()`を元に実装される必要がある
  - 以下の機能は `chat()`を用いると実現できないため
    - tool useの発生時にユーザーに承認を求める機能
    - tool useや推論などの過程を含めたコンテキストの保持
  - 理想的には `completion()`を使って細かなコンテキストの管理やフロー制御を行うべき
    - OpenCode やCodexはこれに近い
- メインとなるエージェントが呼び出すサブエージェントは、`chat()`で実装してもよい
  - サブエージェントの推論過程をメインエージェントに引き継がないという設計の下では可能
  - 渡すtoolsを適切に制限することでユーザー承認がいらないような設計になるなら、`chat()`を使うことで実装が楽になる

## ツール定義

実装を簡略化するため、ツール実行には必ずユーザーへのパーミッションチェックを行う。
またツールの実装全体において、最低限のセキュリティ防護策は取るが、細かな防護策は取らない。

標準ツール
| ツール名 | ファイル | 概要 |
|---------|----------|------|
| `bash`  | `src/tool/bash.rs` | シェルコマンドの実行 |
| `read`  | `src/tool/read.rs` | ファイルの読み取り |
| `write` | `src/tool/write.rs`| ファイル全体を書き込み。新規作成と上書きが可能|
| `grep`  | `src/tool/grep.rs` | 正規表現でファイル内容を検索 |
| `glob`  | `src/tool/grob.rs` | globパターンでファイルを検索 |
| `task`  | `src/tool/task.rs` | サブエージェントの起動。子セッションを作成し、独立したLLM_process()で実行 |

## セッションとログの管理

セッションとは、一つのエージェントが持つ会話履歴を含むコンテキストのことを言います。
基本的にセッションはHashMapを用いて実装され、Microde CLIのプロセス終了を超えて永続化することは考えません。
ただし、実装自体はセッション管理を適切に抽象化することで、今後の拡張(SQLiteを使ってセッション管理を行う、永続化を目指すなど)に備えます。

ログには、ツール実行を含むMicrode CLI全体の状態や実行履歴を記録します。これはLogクレートを使って実装され、標準出力または特定のファイルに書き込まれます。
