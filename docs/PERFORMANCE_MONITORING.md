# パフォーマンス監視

AdBlock はリアルタイムのパフォーマンスメトリクスを提供し、フィルタリングエンジンの効率を監視できます。

## 利用可能なメトリクス

### リクエスト処理
- **総リクエスト数**: 処理したすべてのリクエスト
- **ブロック済みリクエスト**: ブロックされた広告リクエスト
- **許可済みリクエスト**: 通過を許可されたリクエスト
- **ブロック率**: ブロックされたリクエストの割合

### パフォーマンス
- **平均処理時間**: リクエストごとの平均処理時間（マイクロ秒）
- **最大処理時間**: 最も処理に時間がかかったリクエスト
- **最小処理時間**: 最も高速に処理されたリクエスト

### メモリ使用量
- **フィルタ数**: ロードされているフィルタルールの総数
- **メモリ使用量**: エンジンが使用しているメモリ（MB）

### キャッシュ統計
- **キャッシュヒット数**: キャッシュから結果を取得した回数
- **キャッシュミス数**: キャッシュになく計算が必要だった回数
- **キャッシュヒット率**: キャッシュの効率性
- **キャッシュサイズ**: キャッシュエントリ数

### エラー統計
- **パースエラー**: フィルタルールの解析エラー数
- **マッチエラー**: パターンマッチング中のエラー数

## 実装の詳細

### Rust コア

```rust
use adblock_core::metrics::PerformanceMetrics;

// メトリクスの記録
let timer = PerfTimer::start();
let decision = engine.should_block(url);
metrics.record_request(decision.should_block, timer.elapsed());

// メトリクスの取得
let snapshot = metrics.snapshot();
println!("ブロック率: {:.1}%", snapshot.block_rate);
```

### Android

```kotlin
// パフォーマンスメトリクスの取得
val metricsJson = adBlockEngine.getPerformanceMetrics()
val metrics = PerformanceMetrics.fromJson(metricsJson)

// UI での表示
Text("処理時間: ${metrics.getAvgProcessingTimeMicros()} µs")
Text("ブロック率: ${metrics.formatBlockRate()}")
Text("メモリ使用量: ${metrics.getMemoryUsageMB()} MB")
```

### iOS

```swift
// パフォーマンスメトリクスの取得
if let metrics = engine.getPerformanceMetrics() {
    print("処理時間: \(metrics.avgProcessingTimeMicros) µs")
    print("ブロック率: \(metrics.formattedBlockRate)")
    print("メモリ使用量: \(metrics.memoryUsageMB) MB")
}
```

## アーキテクチャ

### メトリクス収集

1. **非同期処理**: アトミック操作を使用してロックフリーで更新
2. **最小オーバーヘッド**: 処理時間の測定は高精度タイマーを使用
3. **スレッドセーフ**: 複数スレッドから安全にアクセス可能

### データ構造

```rust
pub struct PerformanceMetrics {
    // アトミックカウンタ
    total_requests: AtomicU64,
    blocked_requests: AtomicU64,
    
    // 時間統計
    total_processing_time_ns: AtomicU64,
    avg_processing_time_ns: AtomicU64,
    
    // メモリ統計
    filter_count: AtomicUsize,
    memory_usage_bytes: AtomicUsize,
}
```

## パフォーマンス最適化

### Aho-Corasick アルゴリズム
- 複数のドメインパターンを効率的にマッチング
- O(n) の時間複雑度（n はテキスト長）

### キャッシング
- 最近チェックした URL の結果をキャッシュ
- LRU (Least Recently Used) ポリシー

### メモリ効率
- フィルタルールのコンパクトな表現
- 重複排除とパターン共有

## 使用例

### パフォーマンスダッシュボード

```kotlin
@Composable
fun PerformanceCard(metrics: PerformanceMetrics) {
    Card {
        Column(modifier = Modifier.padding(16.dp)) {
            Text("パフォーマンス統計", style = MaterialTheme.typography.headlineSmall)
            
            Spacer(modifier = Modifier.height(8.dp))
            
            Row {
                StatItem("総リクエスト", metrics.totalRequests.toString())
                StatItem("ブロック率", metrics.formatBlockRate())
                StatItem("キャッシュヒット率", metrics.formatCacheHitRate())
            }
            
            Spacer(modifier = Modifier.height(8.dp))
            
            Text("平均処理時間: ${metrics.getAvgProcessingTimeMicros()} µs")
            Text("メモリ使用量: ${metrics.getMemoryUsageMB()} MB")
            Text("フィルタ数: ${metrics.filterCount}")
        }
    }
}
```

### リアルタイム更新

```swift
class PerformanceMonitor: ObservableObject {
    @Published var metrics: PerformanceMetrics?
    private var timer: Timer?
    
    func startMonitoring() {
        timer = Timer.scheduledTimer(withTimeInterval: 1.0, repeats: true) { _ in
            self.updateMetrics()
        }
    }
    
    private func updateMetrics() {
        metrics = AdBlockEngine.shared.getPerformanceMetrics()
    }
}
```

## トラブルシューティング

### 高い処理時間
- フィルタ数が多すぎる可能性
- 複雑な正規表現パターンの確認
- キャッシュサイズの調整

### メモリ使用量が多い
- 不要なフィルタリストの削除
- カスタムルールの最適化
- アプリの再起動

### キャッシュヒット率が低い
- キャッシュサイズを増やす
- URL の正規化を確認
- アクセスパターンの分析

## 今後の拡張予定

- [ ] OpenTelemetry 統合
- [ ] Prometheus メトリクスエクスポート
- [ ] グラフィカルなダッシュボード
- [ ] 異常検知アラート
- [ ] 長期的なトレンド分析