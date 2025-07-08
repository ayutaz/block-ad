# アプリスクリーンショット / App Screenshots

このディレクトリにはApp Store/Google Play用のスクリーンショットを配置します。

## 必要なスクリーンショット

### Android (Google Play)
- 📱 スマートフォン: 1080 x 1920px以上
- 💊 タブレット (7インチ): 1200 x 1920px以上  
- 💊 タブレット (10インチ): 1600 x 2560px以上

各デバイスで2〜8枚

### iOS (App Store)
- 📱 iPhone 6.7": 1290 x 2796px
- 📱 iPhone 6.5": 1284 x 2778px または 1242 x 2688px
- 📱 iPhone 5.5": 1242 x 2208px
- 💊 iPad Pro 12.9": 2048 x 2732px
- 💊 iPad Pro 11": 1668 x 2388px

各デバイスで最大10枚

## 推奨スクリーンショット内容

1. **メイン画面** - VPNトグルと統計情報
2. **ブロック統計** - グラフとブロック数
3. **フィルター設定** - フィルターリスト選択
4. **カスタムルール** - ユーザー定義フィルター
5. **設定画面** - アプリの設定オプション
6. **ブロック通知** - 広告ブロック時の表示

## スクリーンショット生成

実際のアプリが完成後、以下の方法で生成：

### Android
```bash
# Android Studioのエミュレータまたは実機で
adb shell screencap -p /sdcard/screenshot.png
adb pull /sdcard/screenshot.png
```

### iOS
```bash
# Xcodeのシミュレータで
xcrun simctl io booted screenshot screenshot.png
```

## モックアップツール

- [Mockuphone](https://mockuphone.com/)
- [AppMockUp](https://app-mockup.com/)
- [Placeit](https://placeit.net/)

## 注意事項

- 実際の広告コンテンツは表示しない
- ユーザーの個人情報は含めない
- ブランドロゴの無断使用は避ける
- 各ストアのガイドラインに準拠