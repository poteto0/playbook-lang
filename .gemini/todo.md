常にドキュメントをアップデートすること

- [x] 技術選定: 結論
- [x] Core 実装: Basic Canvas (width=100, height=100) & SVG Output
- [x] Core 実装: Lexer (字句解析)
- [x] Core 実装: Parser (構文解析)
- [x] Core 実装: IR (中間表現) 定義
- [x] Core 実装: IR Generator (AST -> IR)
- [x] Core 実装: SVG Renderer (IR -> SVG)
- [x] Wasm 連携設定 (wasm-bindgen)

## 技術選定

### 必須要件

- backend/lang を worker で動かす可能性があり、wasm で動く rust か go または ts を想定する

- ユーザーは golang と typescript に関する知識があるが、rust に関する知識は無いとする

### 非機能要件

- ユーザーの体験を向上させるため、1 ファイル 1 秒以内のコンパイル(レンダリングを目指す)
- そこまでの厳密性は気にする必要はない。
  - 想定外のケースではエラーを返せば必要十分
  - 構文エラーの詳細などは一旦は返す必要なし

### 言語仕様

- playbook-lang/docs/quickStart.md を参照する

### 結論

**Rust**

- **Wasm 親和性**: `wasm-bindgen` により Frontend/Worker 両方で動作する共通ロジックとして実装しやすい。
- **堅牢性**: パーサー実装において型システムが有利。
- **パフォーマンス**: 1 秒以内のレンダリング要件を余裕で満たせる。

### 選定過程

- Rust, Go, TypeScript を比較検討。
- TypeScript は開発速度が早いが、パフォーマンスと Worker での実行効率で劣る可能性がある。
- Go (TinyGo) も Wasm 対応しているが、Rust の `wasm-bindgen` エコシステムの方がブラウザ連携においては成熟している。
- ユーザーは Rust 未経験だが、学習コストを払ってでも品質とパフォーマンスを重視する方向で決定。
