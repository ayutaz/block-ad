# よくある質問（FAQ）

## 目次

1. [一般的な質問](#一般的な質問)
2. [技術的な質問](#技術的な質問)
3. [トラブルシューティング](#トラブルシューティング)
4. [プライバシーに関する質問](#プライバシーに関する質問)
5. [互換性に関する質問](#互換性に関する質問)
6. [パフォーマンスに関する質問](#パフォーマンスに関する質問)

## 一般的な質問

### Q: AdBlockは本当に無料ですか？
**A:** はい、完全無料です。広告表示もアプリ内購入もありません。オープンソースプロジェクトとして開発されています。

### Q: なぜ無料なのですか？
**A:** 私たちは、インターネットをより良い場所にしたいと考えています。広告なしの快適なブラウジング体験は、すべての人が享受すべき権利だと信じています。

### Q: どのような広告をブロックできますか？
**A:** 
- ✅ ウェブサイトの広告（バナー、ポップアップ等）
- ✅ アプリ内広告（多くのアプリ）
- ✅ YouTube広告（80%以上）
- ✅ トラッキングスクリプト
- ❌ 一部のネイティブ広告
- ❌ スポンサードコンテンツ

### Q: YouTube広告を100%ブロックできますか？
**A:** 現在、約80%のYouTube広告をブロックできます。YouTubeは常に広告配信方法を更新しているため、100%のブロックは技術的に困難です。

### Q: データ通信量を節約できますか？
**A:** はい！広告をブロックすることで、月間平均20-40%のデータ通信量を節約できます。

## 技術的な質問

### Q: VPN技術とは何ですか？
**A:** AdBlockは「ローカルVPN」を使用します。これは：
- デバイス内で動作する仮想ネットワーク
- 外部サーバーには接続しません
- すべてのネットワークトラフィックをフィルタリング
- IPアドレスは変更されません

### Q: 他のVPNアプリと併用できますか？
**A:** いいえ、AndroidとiOSは同時に1つのVPNしか使用できません。他のVPNを使用する場合は、AdBlockを一時的に無効にする必要があります。

### Q: フィルターリストとは何ですか？
**A:** ブロックすべき広告のパターンを定義したルールのリストです。AdBlockは以下を使用：
- EasyList（一般的な広告）
- EasyPrivacy（トラッキング防止）
- カスタムYouTubeフィルター

### Q: カスタムルールの書き方は？
**A:** 基本的な形式：
```
||domain.com^           # ドメイン全体をブロック
||ads.*/banner/*       # パターンマッチング
@@||safe.com^          # 例外（許可）
##.ad-class            # CSS要素を非表示
```

### Q: アップデートの頻度は？
**A:** 
- フィルターリスト: 週1回自動更新
- アプリ本体: 月1-2回程度

## トラブルシューティング

### Q: VPNが接続できません
**A:** 以下を確認してください：
1. 他のVPNアプリが動作していないか
2. VPN権限が許可されているか
3. デバイスを再起動
4. アプリを再インストール

### Q: 特定のサイト/アプリが動作しません
**A:** 
1. 一時的にAdBlockを無効にして確認
2. 問題が解決した場合、例外ルールを追加：
   ```
   @@||problem-site.com^
   ```
3. カスタムフィルターで追加

### Q: 広告がブロックされません
**A:** 
1. AdBlockが有効になっているか確認
2. フィルターリストを手動更新
3. アプリ/ブラウザのキャッシュをクリア
4. デバイスを再起動

### Q: アプリがクラッシュします
**A:** 
1. 最新バージョンにアップデート
2. デバイスのストレージ空き容量を確認
3. アプリのデータをクリア（設定は失われます）
4. バグレポートを送信

### Q: バッテリー消費が多い
**A:** 
1. パフォーマンス画面でCPU使用率を確認
2. カスタムルールの数を減らす
3. 自動更新の頻度を下げる
4. 使用しない時は無効にする

## プライバシーに関する質問

### Q: 私のデータは収集されますか？
**A:** いいえ、一切収集しません：
- ❌ 閲覧履歴
- ❌ 個人情報
- ❌ 使用統計
- ❌ IPアドレス
- ✅ すべての処理はローカルで完結

### Q: なぜVPN権限が必要ですか？
**A:** システム全体の広告をブロックするには、すべてのネットワークトラフィックを監視する必要があります。VPN権限はこのために必要です。

### Q: 通信内容は暗号化されますか？
**A:** AdBlockは既存の暗号化を妨げません。HTTPSサイトは引き続き暗号化されます。

### Q: ログは保存されますか？
**A:** いいえ。統計情報（ブロック数等）のみデバイスに保存され、詳細なログは保存されません。

## 互換性に関する質問

### Q: 対応OSバージョンは？
**A:** 
- Android: 7.0（API 24）以上
- iOS: 15.0以上

### Q: タブレットでも使えますか？
**A:** はい、スマートフォンとタブレットの両方で動作します。

### Q: 銀行アプリで問題が発生します
**A:** 一部の銀行アプリはセキュリティ上の理由でVPNを検出します。銀行アプリ使用時は一時的にAdBlockを無効にしてください。

### Q: ゲームアプリへの影響は？
**A:** ほとんどのゲームは正常に動作します。オンラインゲームで問題が発生した場合は、例外ルールを追加してください。

### Q: 動画ストリーミングサービスは？
**A:** Netflix、Amazon Prime、Disney+などは正常に動作します。地域制限には影響しません。

## パフォーマンスに関する質問

### Q: インターネット速度は遅くなりますか？
**A:** ほとんど影響ありません。むしろ広告をブロックすることで、ページ読み込みが高速化することがあります。

### Q: メモリ使用量は？
**A:** 平均20-30MB程度です。最適化により低メモリデバイスでも快適に動作します。

### Q: CPU使用率は？
**A:** 通常1-3%程度。大量のトラフィック処理時でも5%以下です。

### Q: 古いデバイスでも動作しますか？
**A:** はい。2GB RAM以上のデバイスで快適に動作します。

## その他の質問

### Q: 開発に協力できますか？
**A:** もちろん！[GitHub](https://github.com/ayutaz/block-ad)でプルリクエストを歓迎します。

### Q: バグを見つけました
**A:** [GitHub Issues](https://github.com/ayutaz/block-ad/issues)で報告してください。

### Q: 機能リクエストがあります
**A:** [GitHub Discussions](https://github.com/ayutaz/block-ad/discussions)で提案してください。

### Q: 寄付はできますか？
**A:** 現在、寄付は受け付けていません。代わりに：
- プロジェクトにスターを付ける
- 友人に紹介する
- コードに貢献する

### Q: 商用利用できますか？
**A:** はい。MITライセンスの下で自由に使用できます。

---

## 問題が解決しない場合

上記で問題が解決しない場合は：

1. **デバイス情報を含めてバグレポート**
   - デバイスモデル
   - OSバージョン
   - AdBlockバージョン
   - 詳細な症状

2. **スクリーンショット/動画を添付**

3. **[GitHub Issues](https://github.com/ayutaz/block-ad/issues)で報告**

私たちは常にアプリの改善に努めています。フィードバックをお待ちしています！