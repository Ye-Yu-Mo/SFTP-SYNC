<div align="center">

# SFTP-SYNC

**ローカルファイルをSFTPサーバーに自動的に同期するための、モダンでクロスプラットフォームなGUIアプリケーションです。**

</div>

<p align="center">
  <a href="README.md">English</a> | 
  <a href="README.zh-CN.md">简体中文</a> | 
  <a href="README.ja-JP.md">日本語</a> | 
  <a href="README.ko-KR.md">한국어</a>
</p>

<p align="center">
  <img src="https://img.shields.io/github/v/release/Ye-Yu-Mo/SFTP-SYNC?style=for-the-badge" alt="リリースバージョン"/>
  <img src="https://img.shields.io/github/actions/workflow/status/Ye-Yu-Mo/SFTP-SYNC/release.yml?style=for-the-badge" alt="ビルドステータス"/>
  <img src="https://img.shields.io/github/license/Ye-Yu-Mo/SFTP-SYNC?style=for-the-badge" alt="ライセンス"/>
</p>

## 主な機能

- **リアルタイム同期**: ローカルディレクトリの変更を自動的に監視し、即座にアップロードします。
- **モダンなGUI**: [GPUI](https://gpui.dev/)で構築された、高速で直感的、かつGPUアクセラレーション対応のインターフェース。
- **安全な認証情報ストレージ**: パスワードとSSHキーのパスフレーズは、お使いのオペレーティングシステムのネイティブなキーチェーンに安全に保存されます。
- **柔軟な認証方式**: パスワード認証とSSH秘密鍵認証の両方をサポート。
- **複数ターゲット管理**: 複数の同期ターゲットを一つの場所で設定・管理できます。
- **帯域幅制御**: ネットワークリソースを節約するために、アップロード帯域幅を制限するオプションがあります。
- **安全第一**: リモートファイルの削除など、破壊的な操作を実行する前に確認プロンプトを表示します。
- **クロスプラットフォーム**: macOS、Windows、Linuxで動作します。

## インストール

お使いのオペレーティングシステム用の最新リリースは、[**GitHub Releases**](https://github.com/Ye-Yu-Mo/SFTP-SYNC/releases)ページからダウンロードできます。

または、Rustがインストールされている場合は、ソースからビルドすることもできます：

```bash
git clone https://github.com/Ye-Yu-Mo/SFTP-SYNC.git
cd sftp-sync
cargo build --release
```

実行可能ファイルは`target/release`ディレクトリに生成されます。

## 使用方法

1.  **アプリケーションの起動**: SFTP-SYNCを開始します。
2.  **同期ターゲットの追加**:
    - 「Add New Target」をクリックします。
    - SFTPサーバーの詳細情報を入力します：
      - **Target Name**: この接続の分かりやすい名前（例：「My Web Server」）。
      - **Host**: サーバーのアドレス（例：`sftp.example.com`）。
      - **Username**: SFTPのユーザー名。
      - **Authentication**: 「Password」または「SSH Key」を選択します。アプリは認証情報をOSのキーチェーンに安全に保存します。
      - **Local Path**: 同期元のローカルディレクトリ。
      - **Remote Path**: 同期先となるサーバー上の対応するディレクトリ。
3.  **接続**: 作成したターゲットの「Connect」ボタンをクリックします。
4.  **同期開始**: 接続が成功すると、アプリケーションはローカルパスの監視を開始します。新規作成、変更、削除されたファイルはすべて自動的にリモートサーバーにミラーリングされます。

## 設定

アプリケーションの設定は、GUIを通じて直接管理されます。すべての設定と同期ターゲットは、システムの標準設定ディレクトリにある`config.json`ファイルに保存されます。

**グローバル設定（設定パネルで利用可能）:**

- **UI言語**: お好みの言語を選択します。
- **起動時に自動接続**: アプリ起動時にすべてのアクティブなターゲットに自動的に接続します。
- **ローカルの変更を監視**: リアルタイムのファイル監視機能をオン/オフします。
- **破壊的な操作の前に確認**: ファイルを削除する前の安全確認プロンプトを有効/無効にします。
- **帯域幅を制限**: 最大アップロード速度をMbps単位で設定します。

## コントリビューション

貢献を歓迎します！Issueの報告やPull Requestの送信をお気軽にどうぞ。

## ライセンス

このプロジェクトは[MITライセンス](LICENSE)の下でライセンスされています。
