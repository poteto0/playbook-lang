# `core/src/ir/generator.rs` の改善計画 (拡張版)

## 概要
`core/src/ir/generator.rs` の改善について、段階的なアプローチを提案します。
まず「パニック（強制終了）を防ぐ」ことを最優先とし、次に「エラー発生箇所の特定（行番号・列番号の表示）」を実現する計画です。

---

## Phase 1: 基本的なエラーハンドリング (最優先)
**目的:** 不正な入力があってもコンパイラが落ちないようにする。

現状、プレイヤーが存在しない場合に `unwrap_or` で無理やり続行するか、潜在的にパニックする箇所があります。
まずはこれらを `Result` 型で返し、単純なエラーメッセージ（例: "Player p99 not found"）を出せるようにします。

### 手順
1. **テスト作成**: 存在しないプレイヤーを指定したテストケースを追加し、パニックせずにエラーが返ることを確認する。
2. **コード修正**: `unwrap()` や `unwrap_or()` を `ok_or(...)?` に置き換える。
   - 戻り値の型を `Result<Scene, String>` に変更する（既にそうなっていればOK）。

**メリット:**
- 少ない工数で「アプリが落ちる」という致命的な問題を回避できる。
- Rustに詳しくなくても比較的容易に対応可能。

---

## Phase 2: エラー箇所（行・列）の特定 (高度な対応)
**目的:** `Parser` と同様に、`IRGenerator` でも「何行目の何文字目でエラーが起きたか」をユーザーに伝える。

**現状の課題:**
現在、プログラムの中間表現である `AST` (抽象構文木) には、位置情報 (`Span`) が保存されていません。
そのため、`IRGenerator` がデータを受け取った時点では、「p99というプレイヤーがいない」ことは分かっても、「ソースコードのどこに p99 と書かれていたか」は分からなくなっています。

**実現へのステップ (大掛かりな改修が必要です):**

1. **ASTの定義変更 (`core/src/ast/mod.rs`)**
   - 各アクション (`MoveAction`, `PassAction` 等) や識別子に、位置情報を保持するフィールドを追加します。
   ```rust
   use crate::lexer::Span;

   pub struct MoveAction {
       pub span: Span, // 追加: ソースコード上の位置
       pub player: String,
       pub target: (f64, f64),
       // ...
   }
   ```

2. **Parserのロジック変更 (`core/src/parser/mod.rs`)**
   - パース処理中にトークンの `Span` を取得し、AST の構造体にセットするように修正します。
   ```rust
   // イメージ
   let player_token = self.parse_identifier()?;
   let span = player_token.span; // 位置情報を確保
   // ...
   actions.push(MoveAction {
       span,
       player: player_token.value,
       // ...
   });
   ```

3. **Generatorのエラー型変更 (`core/src/ir/generator.rs`)**
   - 文字列 (`String`) ではなく、位置情報を含むエラー型を返すようにします。
   ```rust
   pub struct IRError {
       pub message: String,
       pub span: Span,
   }

   // エラー発生時
   if !positions.contains_key(&player) {
       return Err(IRError {
           message: format!("Player {} not found", player),
           span: action.span, // ASTから位置情報を取り出す
       });
   }
   ```

4. **レンダラーの統合 (`core/src/renderer/mod.rs`)**
   - `Parser` のエラーと `IRGenerator` のエラーを統一的に扱い、JSON形式などでフロントエンドに返す処理を更新します。

### 推奨される進め方
まずは **Phase 1** を完了させ、システムを安定させることが重要です。
その上で、より親切なエラーメッセージが必要であれば **Phase 2** に着手するのが良いでしょう。Phase 2 は AST 全体の見直しになるため、影響範囲が広くなります。