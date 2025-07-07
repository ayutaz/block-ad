# AdBlock for Android

高性能な広告ブロックVPNアプリ

## 🚀 特徴

- ✅ **完全無料** - 広告なし、アプリ内課金なし
- 🛡️ **VPNベース** - システム全体の広告をブロック
- 📱 **YouTube対応** - 動画広告も80%以上ブロック
- ⚡ **軽量・高速** - メモリ使用量30MB以下
- 🔒 **プライバシー重視** - ログ記録なし、データ収集なし

## 📋 動作環境

- Android 7.0 (API 24) 以上
- アーキテクチャ: ARM64、ARMv7、x86、x86_64
- 必要な権限:
  - VPN接続権限
  - インターネットアクセス
  - 通知表示権限（Android 13+）

## 🏗️ ビルド方法

### 必要なツール

- Android Studio Arctic Fox 以降
- JDK 17
- Rust 1.67+
- Android NDK 25.1.8937393
- CMake 3.22.1

### ビルド手順

```bash
# リポジトリのクローン
git clone https://github.com/ayutaz/block-ad.git
cd block-ad/android

# Rust ライブラリのビルド
./build.sh

# Android アプリのビルド
./gradlew assembleDebug
```

### CI/CD

GitHub Actions で自動ビルドが設定されています：

- プッシュ時に自動ビルド
- デバッグ APK とリリース APK を生成
- アーティファクトとしてダウンロード可能

## 📱 インストール

1. [最新のリリース](https://github.com/ayutaz/block-ad/releases)から APK をダウンロード
2. Android デバイスで「不明なソースからのインストール」を許可
3. APK ファイルを開いてインストール

詳細は[インストールガイド](../docs/ANDROID_INSTALLATION.md)を参照してください。

## 🧪 テスト

### ユニットテスト

```bash
./gradlew test
```

### インストルメンテーションテスト

```bash
./gradlew connectedAndroidTest
```

詳細は[テストガイド](../docs/ANDROID_TESTING.md)を参照してください。

## 🏛️ アーキテクチャ

```
android/
├── app/
│   ├── src/
│   │   ├── main/
│   │   │   ├── java/com/adblock/
│   │   │   │   ├── MainActivity.kt      # メイン UI
│   │   │   │   ├── AdBlockEngine.kt     # Rust FFI ラッパー
│   │   │   │   ├── AdBlockVpnService.kt # VPN サービス
│   │   │   │   └── Statistics.kt        # 統計データクラス
│   │   │   └── jniLibs/                 # Rust ライブラリ
│   │   └── test/                        # テストコード
│   └── build.gradle
└── build.sh                             # ビルドスクリプト
```

### 技術スタック

- **UI**: Jetpack Compose + Material Design 3
- **アーキテクチャ**: MVVM
- **非同期処理**: Kotlin Coroutines
- **FFI**: JNI (Java Native Interface)
- **コアエンジン**: Rust

## 🔧 設定

### フィルターリスト

デフォルトで以下の広告ドメインをブロック：
- Google 広告ネットワーク
- Facebook トラッキング
- Amazon 広告
- モバイル広告ネットワーク
- YouTube 広告関連

### カスタマイズ

`AdBlockVpnService.kt` でフィルターリストをカスタマイズ可能：

```kotlin
private fun loadDefaultFilterLists() {
    val customRules = """
        ||custom-ad-domain.com^
        ||another-tracker.net^
    """
    engine.loadFilterList(customRules)
}
```

## 📈 パフォーマンス

- **起動時間**: < 2秒
- **メモリ使用量**: 20-30MB
- **CPU使用率**: < 5%
- **バッテリー影響**: 最小限

## 🤝 貢献方法

1. このリポジトリをフォーク
2. 機能ブランチを作成 (`git checkout -b feature/AmazingFeature`)
3. 変更をコミット (`git commit -m 'Add some AmazingFeature'`)
4. ブランチをプッシュ (`git push origin feature/AmazingFeature`)
5. プルリクエストを作成

## 📄 ライセンス

MIT License - 詳細は[LICENSE](../LICENSE)を参照

## 🙏 謝辞

- [Rust](https://www.rust-lang.org/)
- [Jetpack Compose](https://developer.android.com/jetpack/compose)
- [Aho-Corasick](https://github.com/BurntSushi/aho-corasick) アルゴリズム