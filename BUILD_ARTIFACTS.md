# ビルド成果物のダウンロード方法

## GitHub Actionsからのダウンロード

### Android APK

1. [Actions](https://github.com/YOUR_USERNAME/block-ad/actions)ページを開く
2. 最新の「Android CI」ワークフローをクリック
3. ページ下部の「Artifacts」セクションから以下をダウンロード：
   - **AdBlock-debug-[番号]**: デバッグ版APK（すぐにインストール可能）
   - **AdBlock-release-unsigned-[番号]**: リリース版APK（署名が必要）

#### インストール方法
1. Androidデバイスの「設定」→「セキュリティ」で「提供元不明のアプリ」を有効化
2. ダウンロードしたデバッグAPKをデバイスに転送
3. ファイルマネージャーでAPKをタップしてインストール

### iOS IPA

1. [Actions](https://github.com/YOUR_USERNAME/block-ad/actions)ページを開く
2. 最新の「iOS CI」ワークフローをクリック
3. ページ下部の「Artifacts」セクションから以下をダウンロード：
   - **AdBlock-iOS-Simulator-[番号]**: シミュレーター用ビルド
   - **AdBlock-iOS-unsigned-[番号]**: デバイス用IPA（署名が必要）
   - **AdBlock-iOS-xcarchive-[番号]**: 再署名用アーカイブ

#### インストール方法
- **シミュレーター**: Xcodeでシミュレーターを起動し、.appファイルをドラッグ&ドロップ
- **実機**: Apple Developer証明書で署名後、XcodeまたはApple Configuratorでインストール

## 手動ビルド

### Android
```bash
cd android
./gradlew assembleDebug
# APKは android/app/build/outputs/apk/debug/app-debug.apk に生成されます
```

### iOS
```bash
cd ios
./build.sh
xcodebuild -scheme AdBlock -sdk iphonesimulator build
```

## ワークフローの手動実行

1. GitHubの[Actions](https://github.com/YOUR_USERNAME/block-ad/actions)ページを開く
2. 左サイドバーから「Android CI」または「iOS CI」を選択
3. 「Run workflow」ボタンをクリック
4. ブランチを選択して「Run workflow」を実行

ビルドが完了すると、成果物が自動的にArtifactsとして保存されます。