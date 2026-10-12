<div align="center">

<img src="assets/icon.svg" width="72" height="72" alt="pp-switch icon">

# pp-switch

**Pure Provider Switch. 設定は、あなたのまま。**

Claude Code・Codex・Gemini CLI の API URL とキーを切り替える小さなアプリ。

[![CI](https://github.com/phonowell/pp-switch/actions/workflows/ci.yml/badge.svg)](https://github.com/phonowell/pp-switch/actions/workflows/ci.yml)
[![MIT](https://img.shields.io/badge/license-MIT-465bab.svg)](LICENSE)
![Platforms](https://img.shields.io/badge/macOS%20%7C%20Windows%20%7C%20Linux-native-465bab)

**[English](README.md) · [简体中文](README_ZH.md) · 日本語 · [Deutsch](README_DE.md)**

[ダウンロード](https://github.com/phonowell/pp-switch/releases) · [ガイド](docs/user-guide-ja.md) · [実測結果](docs/verification.md) · [仕様](SPEC.md)

</div>

![クライアントタブ、プロバイダー一覧、切り替えボタンを表示する日本語のネイティブ UI](assets/screenshot-ja.png)

## pp-switch を選ぶ理由

**API サービスを変えるたびに、調整済みのクライアントを設定し直したくない。**

モデル、推論設定、プロジェクト権限、MCP、Skills はすでに使いやすく整っています。変えたいのは URL とキーだけ。pp-switch は接続を保存し、切り替え時にその二つの値だけを変更します。無関係な設定のバイト列、コメント、書式は維持します。非対応の設定は明示的に拒否し、勝手に作り直しません。

**使い終わったら、終了する。** 一つのウィンドウ。トレイ、スタートアップ常駐、プロバイダーへの疎通確認、使用量の定期取得はありません。閉じるとプロセスが終了します。更新も「更新を確認」を押してインストールに同意した場合だけ。Rust + FLTK、ネイティブのベクター描画とシステムフォントを使い、WebView・データベース・JavaScript ランタイムを同梱しません。

[cc-switch #7649](https://github.com/farion1231/cc-switch/issues/7649) でも Pure/Lite モードと「管理対象以外の設定はアプリやユーザーに任せる」ことが求められています。pp-switch はこの設定の所有原則を URL とキーに限定して実装します。議論は着想の根拠であり、推薦を意味しません。[調査資料と留意点](docs/verification.md#community-sources)。

## 小さな範囲、明確な違い

| | pp-switch | cc-switch |
| --- | --- | --- |
| 目的 | 設定済みクライアントの URL / キー切り替え | より広いプロバイダー・クライアント管理 |
| 書き込み | 現在有効な Base URL と API キーのみ | 有効な機能に応じてより多くの設定 |
| リンク | 対象外 | 完全なディープリンク機能 |
| モデル、ルーティング、使用量、MCP、プロンプト、Skills | 製品の範囲外 | 対応する管理機能 |
| 終了 | ウィンドウを閉じるとプロセス終了 | トレイなどのワークフロー |
| 更新 | 手動確認 → 同意 → ダウンロード・インストール・再起動 | 独自の更新機能 |

総合管理には [cc-switch](https://github.com/farion1231/cc-switch) が適しています。pp-switch は独立したプロジェクトで、設定済みのクライアントを小さく予測可能な操作で切り替えたい人向けです。

## ダウンロード

**0.0.4 はレビュー候補で、タグ作成・公開はまだ行っていません。** [検証結果](docs/verification.md#review-candidate-004)を参照してください。以下のダウンロードは公開済みの 0.0.3 プレビューです。

**0.0.4 の UI 改訂は 2026-10-12 にユーザー承認済みです。公開は承認済みで、ネイティブ公開 CI は未完了です。** 中国語のネイティブ操作と全言語の外観を別々に検証。[確認記録](docs/smoke.md)。

**[v0.0.3 機能プレビュー](https://github.com/phonowell/pp-switch/releases/tag/v0.0.3)**。macOS のメモリは予算超過です。[実測結果](docs/verification.md) を公開しています。

| OS | パッケージ | 実行 |
| --- | --- | --- |
| macOS · Apple Silicon | [ZIP](https://github.com/phonowell/pp-switch/releases/download/v0.0.3/pp-switch-v0.0.3-aarch64-apple-darwin.zip) | 展開し `.app` を Applications へ |
| macOS · Intel | [ZIP](https://github.com/phonowell/pp-switch/releases/download/v0.0.3/pp-switch-v0.0.3-x86_64-apple-darwin.zip) | 同上 |
| Windows · x64 | [ZIP](https://github.com/phonowell/pp-switch/releases/download/v0.0.3/pp-switch-v0.0.3-x86_64-pc-windows-msvc.zip) | 展開し `pp-switch.exe` を実行 |
| Linux · x64 | [tar.gz](https://github.com/phonowell/pp-switch/releases/download/v0.0.3/pp-switch-v0.0.3-x86_64-unknown-linux-gnu.tar.gz) | 展開しデスクトップで `pp-switch` を実行 |

Rust / Node.js のインストールは不要。macOS は 11+ を宣言していますが、**Apple Developer 署名・公証はありません**。Windows は 10+。Linux は X11/XWayland、X11/fontconfig ライブラリ、選択した言語を表示できるシステムフォントが必要です。宣言された最小 OS はすべてのバージョンの動作確認を意味しません。[検証環境](docs/verification.md) を参照してください。

macOS 0.0.2 からは今回だけ手動ダウンロードが必要です。旧版は追加のアイコン資源を拒否します。0.0.3 の更新対象にはアイコンを含めています。

## 使い方

1. Claude Code、Codex、Gemini CLI を選択。
2. 「追加」で名前、Base URL、キーを入力し保存。
3. 各行の編集アイコン右側にある切り替えを押す。必要ならクライアントを再起動。

各行の編集・切り替えは Lucide アイコンです。行の選択だけでは適用しません。
タブ右側の小さな歯車から設定へ進み、言語と手動更新を管理します。
すべてのツールチップ、画面内タイトル、下部の状態・切り替え領域とリンク取り込みを削除。
一致する重複項目はすべて「使用中」です。システムのライト／ダーク設定に追従。
保存はローカル一覧のみを変更。UI は **English・简体中文・繁體中文・日本語**で、初回はシステム言語。


| クライアント | 必要な既存設定 | 変更する値 |
| --- | --- | --- |
| Claude Code | 外部 key helper、クラウドモード、競合する認証情報のない API キー設定 | `settings.json` の `ANTHROPIC_BASE_URL` と既存キー/token |
| Codex | カスタムプロバイダー選択済み、API キー認証設定済み | 選択プロバイダーの `base_url` と既存 bearer token または `auth.json.OPENAI_API_KEY` |
| Gemini CLI | `/auth` で API キーモード選択済み | `.env` の `GOOGLE_GEMINI_BASE_URL` と `GEMINI_API_KEY` |

モデル・認証方式・プロトコルの変更や初期設定はしません。ユーザー設定だけが対象で、プロジェクト設定・起動引数・別ターミナルの環境変数が優先される場合があります。`CLAUDE_CONFIG_DIR`、`CODEX_HOME` に対応。[詳細・トラブル対処 →](docs/user-guide-ja.md)

## 互換性と信頼

- 現在の候補版はリンクの取り込みと URL 登録を提供しません。保存済み項目は保持します。
- 秘密はローカルのユーザー専用ファイル（Unix 0600 / Windows 現在ユーザー ACL）に保存。マスク表示は**暗号化ではありません**。自動クリップボード読取、秘密の診断出力、プロバイダーへの通信はありません。
- 更新は手動確認と書き込み可能な設置先が必要です。本リポジトリのパッケージだけを取得し、SHA-256 を検証します。パッケージは未署名。プロバイダー・クライアント設定は対象外です。[更新の詳細](docs/user-guide-ja.md#手動更新)。
- 完全な動作契約は [SPEC.md](SPEC.md)、保存先は[ガイド](docs/user-guide-ja.md)。

## 性能を公開する

**0.0.3 ネイティブ CI 実測：**

| ネイティブ CI | 容量 MiB | アイドル RSS MiB | スクリプト予算 |
| --- | ---: | ---: | --- |
| macOS 15 · ARM64 | 2.51 | 66.64 | 未達 |
| macOS 15 · Intel | 2.73 | 38.40 | 未達 |
| Windows Server 2022 · x64 | 2.81 | 15.00 | 合格 |
| Ubuntu 22.04 · x64 · Xvfb | 3.02 | 17.19 | 合格 |

予算の定義は [SPEC PP-11](SPEC.md#acceptance-contract)。Windows / Linux は
このスクリプトに合格し、macOS のメモリは予算超過です。厳密なコールド起動、
インポート・更新のピークは未検証です。[測定方法・生データ・過去の結果](docs/verification.md)。

## 開発

[SPEC.md](SPEC.md) を先に確認してください。ビルド・検証・リリース手順は
[貢献ガイド（英語）](CONTRIBUTING.md) にまとめています。

## 謝辞とライセンス

[cc-switch](https://github.com/farion1231/cc-switch) の公開プロトコル・実装資料、[fltk-rs](https://github.com/fltk-rs/fltk-rs) / [FLTK](https://www.fltk.org) に感謝します。cc-switch とは独立したプロジェクトです。[MIT](LICENSE)。依存の[ライセンスと通知](THIRD_PARTY_NOTICES.txt) を保持します。
