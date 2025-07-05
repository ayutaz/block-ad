# AdBlocker - 完全無料の広告ブロックアプリ

Android/iOS対応の軽量・高速な広告ブロックアプリケーション。YouTube広告にも対応。

## 特徴

- ✅ **完全無料** - 広告なし、課金なし、オープンソース
- ✅ **YouTube広告ブロック** - 動画広告も効果的にブロック
- ✅ **軽量・高速** - メモリ使用量30MB以下、バッテリー効率的
- ✅ **プライバシー重視** - ログ収集なし、全処理をローカルで実行
- ✅ **クロスプラットフォーム** - Android/iOS両対応

## 技術仕様

- **コアエンジン**: Rust（高速パターンマッチング）
- **Android**: Kotlin + VPN Service
- **iOS**: Swift + Network Extension
- **最小要件**:
  - Android 7.0以上
  - iOS 15.0以上

## ドキュメント

### 設計ドキュメント
- [技術設計書](./TECHNICAL_DESIGN.md) - 詳細な技術仕様
- [アーキテクチャ設計](./ARCHITECTURE.md) - システム構成
- [API設計](./API_DESIGN.md) - インターフェース仕様
- [実装ガイド](./IMPLEMENTATION_GUIDE.md) - 開発手順

### 仕様・テストドキュメント
- [アプリ仕様書](./SPECIFICATION.md) - 機能仕様と要件定義
- [テスト項目書](./TEST_SPECIFICATION.md) - テスト計画と項目
- [TDDガイド](./TDD_GUIDE.md) - テスト駆動開発の手順

### プロジェクト管理
- [プロジェクトステータス](./PROJECT_STATUS.md) - 現在の進捗と次のステップ

## クイックスタート

### Android
```bash
# APKダウンロード（リリース後）
# GitHub Releasesから最新版をダウンロード
```

### iOS
```bash
# TestFlight経由（申請後）
# またはIPAファイルをサイドロード
```

## ビルド方法

### 前提条件
- Rust 1.70+
- Android Studio / Xcode
- 各プラットフォームのSDK

### ビルド手順
```bash
# リポジトリクローン
git clone https://github.com/yourusername/block-ad.git
cd block-ad

# 依存関係インストール
./scripts/setup.sh

# ビルド
./scripts/build_all.sh
```

## 開発ロードマップ

- [x] 技術設計
- [ ] Phase 1: Rustコア実装
- [ ] Phase 2: Android MVP
- [ ] Phase 3: YouTube対応強化  
- [ ] Phase 4: iOS版リリース

## ライセンス

MIT License - 完全に自由に使用・改変可能

## 貢献

プルリクエスト歓迎！詳細は[CONTRIBUTING.md](./CONTRIBUTING.md)を参照。