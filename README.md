# AdBlock - Free & Open Source Ad Blocker

[![Rust Core CI](https://github.com/ayutaz/block-ad/actions/workflows/rust-core.yml/badge.svg)](https://github.com/ayutaz/block-ad/actions/workflows/rust-core.yml)
[![Android CI](https://github.com/ayutaz/block-ad/actions/workflows/android.yml/badge.svg)](https://github.com/ayutaz/block-ad/actions/workflows/android.yml)
[![iOS CI](https://github.com/ayutaz/block-ad/actions/workflows/ios.yml/badge.svg)](https://github.com/ayutaz/block-ad/actions/workflows/ios.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

完全無料・オープンソースの広告ブロッカーアプリです。Android/iOS両対応で、YouTube広告を含むあらゆる広告をブロックします。

## 🚀 特徴

- ✅ **完全無料** - 広告なし、アプリ内購入なし
- ✅ **YouTube広告ブロック** - 80%以上のブロック率
- ✅ **軽量・高速** - メモリ使用量30MB以下
- ✅ **クロスプラットフォーム** - Android/iOS両対応
- ✅ **オープンソース** - 完全に透明性のあるコード
- ✅ **プライバシー重視** - データ収集なし

## 📱 対応プラットフォーム

- Android 7.0 (API 24) 以上
- iOS 15.0 以上

## 🏗️ アーキテクチャ

```
block-ad/
├── core/          # Rust製の高速フィルタリングエンジン
├── android/       # Android VPNサービス実装
└── ios/          # iOS Network Extension実装
```

### 技術スタック

- **Core**: Rust (Aho-Corasick algorithm)
- **Android**: Kotlin, Jetpack Compose, VPN Service
- **iOS**: Swift, SwiftUI, Network Extension
- **CI/CD**: GitHub Actions

## 📥 インストール

### Android

#### 方法1: GitHub Actions から最新版をダウンロード（推奨）
1. [Android CI](https://github.com/ayutaz/block-ad/actions/workflows/android.yml)にアクセス
2. 最新の成功したビルドを選択
3. Artifactsセクションから`debug-apk`をダウンロード
4. APKをインストール（不明なソースからのインストールを許可する必要があります）
5. アプリを起動してVPNを有効化

#### 方法2: コマンドラインでダウンロード
```bash
# GitHub CLIを使用
gh run list --workflow=android.yml --status=completed --limit=1
gh run download <RUN_ID> --name debug-apk
```

詳細は[APKダウンロードガイド](DOWNLOAD_APK.md)を参照してください。

### iOS

1. [Releases](https://github.com/ayutaz/block-ad/releases)から最新のIPAをダウンロード
2. AltStore/Sideloadlyなどを使用してインストール
3. 設定 > 一般 > VPNとデバイス管理 で信頼
4. アプリを起動してVPNを有効化

## 🛠️ ビルド方法

### 必要な環境

- Rust 1.70+
- Android Studio (Android)
- Xcode 15+ (iOS)
- cargo-ndk (Android)

### Android

```bash
cd android
./build.sh
```

### iOS

```bash
cd ios
./build.sh
```

## 🧪 テスト

```bash
# Rustコアテスト
cargo test

# Androidテスト
cd android && ./gradlew test

# iOSテスト
cd ios && xcodebuild test -scheme AdBlock
```

## 📊 パフォーマンス

- **メモリ使用量**: < 30MB
- **YouTube広告ブロック率**: 80%+
- **起動時間**: < 1秒
- **フィルタリング遅延**: < 1ms

## 🤝 コントリビューション

プルリクエストを歓迎します！以下のガイドラインに従ってください：

1. Issueを作成して議論
2. Forkしてfeatureブランチを作成
3. テストを追加
4. CI/CDがパスすることを確認
5. プルリクエストを送信

詳細は[CONTRIBUTING.md](CONTRIBUTING.md)を参照してください。

## 📝 ライセンス

このプロジェクトはMITライセンスの下で公開されています。詳細は[LICENSE](LICENSE)ファイルを参照してください。

## 🔒 プライバシーポリシー

- ユーザーデータの収集なし
- サードパーティへのデータ送信なし
- 完全にローカルで動作
- 詳細は[PRIVACY.md](PRIVACY.md)を参照

## 🆘 サポート

- [Issues](https://github.com/ayutaz/block-ad/issues) - バグ報告・機能要望
- [Discussions](https://github.com/ayutaz/block-ad/discussions) - 質問・議論

## 📚 ドキュメント

### 開発ドキュメント
- [開発状況](./docs/development/STATUS.md) - 現在の実装状況
- [実装ガイド](./docs/development/IMPLEMENTATION_GUIDE.md) - 開発手順
- [TDDガイド](./docs/development/TDD_GUIDE.md) - テスト駆動開発の手順

### 設計ドキュメント
- [アプリ仕様書](./docs/design/SPECIFICATION.md) - 機能仕様と要件定義
- [技術設計書](./docs/design/TECHNICAL_DESIGN.md) - 詳細な技術仕様
- [アーキテクチャ設計](./docs/design/ARCHITECTURE.md) - システム構成
- [API設計](./docs/design/API_DESIGN.md) - インターフェース仕様

### テストドキュメント
- [テスト仕様書](./docs/testing/TEST_SPECIFICATION.md) - テスト計画と項目

## 🏆 謝辞

- EasyListコミュニティ
- Rust Aho-Corasickライブラリ
- すべてのコントリビューター

---

Made with ❤️ by [ayutaz](https://github.com/ayutaz) and contributors