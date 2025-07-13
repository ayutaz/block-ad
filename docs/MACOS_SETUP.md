# macOS VPN サポート設定ガイド

このガイドでは、AdBlock の macOS VPN サポートを有効にする方法を説明します。

## 必要な設定

### 1. Xcode でのターゲット追加

1. Xcode で `AdBlock.xcodeproj` を開く
2. File → New → Target を選択
3. macOS → Network Extension を選択
4. Product Name: `AdBlockNetworkExtension-macOS`
5. Language: Swift を選択
6. Bundle Identifier: `com.adblock.app.networkextension`

### 2. ビルド設定

#### メインアプリ (macOS)
- **Bundle Identifier**: `com.adblock.app`
- **Deployment Target**: macOS 11.0
- **Entitlements**: `AdBlock-macOS.entitlements`

#### Network Extension
- **Bundle Identifier**: `com.adblock.app.networkextension`
- **Deployment Target**: macOS 11.0
- **Entitlements**: `AdBlockNetworkExtension-macOS.entitlements`

### 3. Capabilities 設定

#### メインアプリ
1. Signing & Capabilities タブを開く
2. + Capability をクリック
3. 以下を追加:
   - App Groups (ID: `group.com.adblock.app`)
   - Network Extensions
   - System Extension

#### Network Extension
1. Signing & Capabilities タブを開く
2. + Capability をクリック
3. 以下を追加:
   - App Groups (ID: `group.com.adblock.app`)
   - Network Extensions (Packet Tunnel Provider)

### 4. Info.plist 設定

メインアプリの Info.plist に以下を追加:

```xml
<key>NSSystemExtensionUsageDescription</key>
<string>AdBlock は広告をブロックするためにネットワーク拡張機能を使用します。</string>
```

### 5. ビルドフェーズ設定

1. メインアプリのターゲットを選択
2. Build Phases → Dependencies に Network Extension ターゲットを追加
3. Build Phases → Embed System Extensions を追加
4. Network Extension を追加

## 実装の詳細

### PacketTunnelProvider

`PacketTunnelProvider` クラスは以下の機能を提供します:

1. **DNS フィルタリング**: DNS クエリをインターセプトし、広告ドメインをブロック
2. **IP フィルタリング**: 既知の広告サーバー IP への接続をブロック
3. **統計情報**: ブロックしたリクエスト数の追跡

### VPNManager

macOS 版の `VPNManager` は以下を実装:

1. System Extension API を使用した VPN 設定
2. NETunnelProviderManager による接続管理
3. アプリと Extension 間の通信

## テスト方法

### 開発環境でのテスト

1. System Extension の開発には SIP (System Integrity Protection) の無効化が必要な場合があります
2. 開発用 Mac で以下を実行:
   ```bash
   systemextensionctl developer on
   ```

### テスト手順

1. アプリをビルドして実行
2. VPN 接続を有効化
3. System Preferences → Network で VPN 接続を確認
4. ブラウザで広告を含むサイトにアクセスして動作確認

## トラブルシューティング

### "System extension blocked" エラー

1. System Preferences → Security & Privacy を開く
2. General タブで "Allow" をクリック
3. アプリを再起動

### VPN が接続されない

1. Console.app でログを確認
2. `com.adblock.networkextension` でフィルタリング
3. エラーメッセージを確認

### 権限エラー

1. アプリと Extension の Bundle ID が正しいか確認
2. Entitlements ファイルが正しく設定されているか確認
3. App Groups が両方のターゲットで同じか確認

## リリース時の注意

1. **Developer ID 証明書**: System Extension には Developer ID が必要
2. **Notarization**: アプリは Apple による公証が必要
3. **配布**: Mac App Store では System Extension は配布できないため、直接配布のみ

## 参考リンク

- [Network Extension Framework](https://developer.apple.com/documentation/networkextension)
- [System Extensions](https://developer.apple.com/documentation/systemextensions)
- [Packet Tunnel Provider](https://developer.apple.com/documentation/networkextension/nepackettunnelprovider)