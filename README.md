# defaults-util

macOSの`defaults`コマンドによる設定変更を検出し、再現可能なコマンドを生成するTUIアプリケーション。

## 概要

System Settingsで設定を変更した後、その変更を`defaults write`コマンドとして取得できます。dotfilesへの組み込みや設定の再現に便利です。

## インストール

```bash
cargo build --release
```

## 制限事項

- macOS専用
- 一部のドメインは読み取り権限がない場合があります（スキップされます）
- クリップボードへのコピーは`pbcopy`を使用

## ライセンス

MIT
