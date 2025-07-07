# Android APK ダウンロードガイド

## 📱 最新のAndroid APKをダウンロードする方法

### 方法1: GitHub Actions から直接ダウンロード（推奨）

1. **GitHub Actions ページにアクセス**
   - https://github.com/ayutaz/block-ad/actions/workflows/android.yml

2. **最新の成功したビルドを選択**
   - 緑色のチェックマークがついているものを選択
   - または "Android CI" ワークフローの最新の実行を選択

3. **Artifacts セクションからダウンロード**
   - `debug-apk` - テスト用（すぐにインストール可能）
   - `release-apk` - リリース用（署名が必要）

### 方法2: GitHub CLI を使用

```bash
# 最新の成功したビルドIDを確認
gh run list --workflow=android.yml --status=completed --limit=1

# APKをダウンロード（例: Run ID 16099974399）
gh run download 16099974399 --name debug-apk
```

### 方法3: 直接リンク（最新の成功ビルド）

最新の成功したビルド（Run #16099974399）：
- **ビルド日時**: 2025-07-06
- **ファイルサイズ**: 
  - Debug APK: 14.9MB
  - Release APK: 10.3MB

## 📦 APKの種類

### Debug APK (`app-debug.apk`)
- **用途**: テスト・開発用
- **署名**: デバッグ証明書で署名済み
- **インストール**: そのままインストール可能
- **推奨**: 一般ユーザー向け

### Release APK (`app-release-unsigned.apk`)
- **用途**: 本番配布用
- **署名**: 未署名（配布前に署名が必要）
- **インストール**: 署名後にインストール可能
- **推奨**: 開発者・配布者向け

## 🚀 インストール方法

1. **Androidデバイスの設定**
   - 設定 → セキュリティ → 不明なソースからのインストールを許可

2. **APKファイルをデバイスに転送**
   - ダウンロードしたAPKをAndroidデバイスにコピー

3. **インストール**
   - ファイルマネージャーでAPKをタップ
   - インストールボタンをタップ

## ⚠️ 注意事項

- Android 7.0（API 24）以上が必要
- 初回起動時にVPN権限の許可が必要
- 他のVPNアプリとの同時使用不可