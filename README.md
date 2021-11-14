# HACK TO THE FUTURE qual

[HACK TO THE FUTURE 予選](https://atcoder.jp/contests/future-contest-2022-qual) に提出したコードです (暫定テスト 89000 点程度)。

## テスト用スクリプト

手元で複数ケースを回すために `runner.py` というスクリプトを書きました。実行後、スコア総合計と、50ケース換算のスコアを出力します。

-   Python 3.9
-   ローカルテスターを設置ずみ ([問題ページ](https://atcoder.jp/contests/future-contest-2022-qual/tasks/future_contest_2022_qual_a) からダウンロード可能)

引数は、`実行ファイル` ・ `ローカルテスターのディレクトリ` ・ `入力ファイルが入ったディクトリ` ・ `出力ファイルを格納するディレクトリ` です。 `--verbose` オプションで標準エラー出力を表示します。

以下、実行例。

```bash
cargo build --release && python runner.py target/release/future-contest-2022-qual-a ../future-contest-2022-qual-a-tools ../future-contest-2022-qual-a-tools/in ../future-contest-2022-qual-a-tools/out
```
