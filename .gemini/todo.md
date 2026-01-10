- [x] コートを描画する
  - [x] 線を補足する
    - [x] コートは 3on3 のように片方のコートの 3P ライン付近だけを書く
    - [x] 上中央に、赤色でゴールリングを書く
    - [x] 黒色で 3 ポイントラインと、フリースローラインを書く

## Compile Error Implementation
- [x] エラーハンドリングの改善
    - [x] Lexer: Tokenに位置情報(Span)を含める
    - [x] Parser: エラー時に位置情報を報告する
    - [x] Renderer: エラーメッセージをテキストとして返す (SVGではない)
    - [x] Typo修正提案機能 (Levenshtein distance <= 2)

## Screen Action Enhancement
- [x] Screenアクションで座標指定を可能にする
    - [x] AST: `ScreenAction`の`target`を`ScreenTarget` enumに変更 (Player or Coordinate)
    - [x] Parser: `screen`アクションで座標とプレイヤー名の両方をパースできるように変更
    - [x] IR: `ScreenTarget`に応じて座標を解決するように変更
    - [x] Test: テストケースの追加