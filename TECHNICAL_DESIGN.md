# 広告ブロックアプリ 技術設計書

## 1. プロジェクト概要

### 1.1 目的
- Android、iOSで動作する完全無料の広告ブロックアプリケーション
- Webサイトおよび動画（YouTube含む）の広告を完全にブロック
- 軽量・高速な動作を実現

### 1.2 主要機能
- ✅ 全プラットフォーム対応（Android/iOS）
- ✅ YouTube広告ブロック対応（80%以上のブロック率）
- ✅ 完全無料・オープンソース
- ✅ 軽量高速（メモリ使用量 < 30MB）
- ✅ バッテリー効率的な動作（24時間で3%以下の消費）

## 2. システムアーキテクチャ

### 2.1 全体構成
```
┌─────────────────────────────────────────────────────┐
│                   User Interface                     │
│  ┌─────────────┐              ┌─────────────────┐  │
│  │   Android   │              │      iOS        │  │
│  │   (Kotlin)  │              │    (Swift)      │  │
│  └──────┬──────┘              └────────┬────────┘  │
│         │                              │            │
├─────────┴──────────────────────────────┴────────────┤
│                  Platform Bridge                     │
│                    (Rust FFI)                        │
├──────────────────────────────────────────────────────┤
│                   Core Engine                        │
│                     (Rust)                           │
│  ┌────────────┐  ┌──────────┐  ┌───────────────┐  │
│  │  Filter    │  │ Network  │  │   Pattern     │  │
│  │  Engine    │  │Interceptor│ │   Matcher     │  │
│  └────────────┘  └──────────┘  └───────────────┘  │
└──────────────────────────────────────────────────────┘
```

### 2.2 コンポーネント詳細

#### 2.2.1 Core Engine (Rust)
- **役割**: 高速フィルタリング処理の中核
- **主要機能**:
  - URLパターンマッチング
  - コンテンツフィルタリング
  - ルール管理・更新

#### 2.2.2 Platform Bridge
- **役割**: Rust CoreとネイティブUIの橋渡し
- **実装**: Foreign Function Interface (FFI)

#### 2.2.3 Android Layer
- **VPN Service**: ローカルVPNとして動作
- **UI**: Material Design 3準拠

#### 2.2.4 iOS Layer
- **Network Extension**: システムレベルフィルタリング
- **Safari Extension**: ブラウザレベルフィルタリング

## 3. 技術スタック

### 3.1 共通コア
- **言語**: Rust 1.70+
- **フィルタリングエンジン**: 
  - aho-corasick (高速パターンマッチング)
  - regex (正規表現)
- **ネットワーク**: tokio (非同期I/O)
- **データ形式**: Protocol Buffers

### 3.2 Android
- **言語**: Kotlin 1.9+
- **最小SDK**: API 24 (Android 7.0)
- **ビルドシステム**: Gradle 8.0+
- **依存関係**:
  ```gradle
  dependencies {
      implementation "org.jetbrains.kotlinx:kotlinx-coroutines-android:1.7.0"
      implementation "androidx.lifecycle:lifecycle-viewmodel-ktx:2.6.0"
      implementation "com.google.android.material:material:1.9.0"
  }
  ```

### 3.3 iOS
- **言語**: Swift 5.8+
- **最小バージョン**: iOS 15.0+
- **フレームワーク**:
  - NetworkExtension
  - SafariServices
  - SwiftUI

## 4. 実装詳細

### 4.1 フィルタリングエンジン

#### 4.1.1 Rustコア実装
```rust
// src/filter_engine.rs
use aho_corasick::{AhoCorasick, PatternID};
use std::sync::Arc;

pub struct FilterEngine {
    // URLパターンマッチャー
    url_matcher: Arc<AhoCorasick>,
    // CSSセレクタルール
    css_rules: Vec<CssRule>,
    // スクリプトインジェクションルール
    script_rules: Vec<ScriptRule>,
}

impl FilterEngine {
    pub fn new(rules: &[FilterRule]) -> Self {
        let patterns: Vec<&str> = rules
            .iter()
            .filter_map(|r| match r {
                FilterRule::UrlBlock(pattern) => Some(pattern.as_str()),
                _ => None,
            })
            .collect();
        
        let url_matcher = Arc::new(
            AhoCorasick::builder()
                .auto_configure(&patterns)
                .build(&patterns)
                .unwrap()
        );
        
        FilterEngine {
            url_matcher,
            css_rules: extract_css_rules(rules),
            script_rules: extract_script_rules(rules),
        }
    }
    
    pub fn should_block_url(&self, url: &str) -> bool {
        self.url_matcher.is_match(url)
    }
    
    pub fn get_css_rules_for_domain(&self, domain: &str) -> Vec<String> {
        self.css_rules
            .iter()
            .filter(|rule| rule.matches_domain(domain))
            .map(|rule| rule.selector.clone())
            .collect()
    }
}

#[derive(Debug, Clone)]
pub enum FilterRule {
    UrlBlock(String),
    CssHide { selector: String, domains: Vec<String> },
    ScriptInject { script: String, domains: Vec<String> },
}
```

#### 4.1.2 FFIブリッジ
```rust
// src/ffi.rs
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn filter_engine_new(rules_json: *const c_char) -> *mut FilterEngine {
    let rules_str = unsafe { CStr::from_ptr(rules_json).to_string_lossy() };
    let rules: Vec<FilterRule> = serde_json::from_str(&rules_str).unwrap();
    Box::into_raw(Box::new(FilterEngine::new(&rules)))
}

#[no_mangle]
pub extern "C" fn filter_engine_should_block(
    engine: *const FilterEngine,
    url: *const c_char,
) -> bool {
    let engine = unsafe { &*engine };
    let url = unsafe { CStr::from_ptr(url).to_string_lossy() };
    engine.should_block_url(&url)
}

#[no_mangle]
pub extern "C" fn filter_engine_free(engine: *mut FilterEngine) {
    if !engine.is_null() {
        unsafe { Box::from_raw(engine) };
    }
}
```

### 4.2 Android実装

#### 4.2.1 VPN Service
```kotlin
// VpnService.kt
package com.adblocker.android.service

import android.app.PendingIntent
import android.content.Intent
import android.net.VpnService
import android.os.ParcelFileDescriptor
import kotlinx.coroutines.*
import java.nio.ByteBuffer
import java.nio.channels.DatagramChannel

class AdBlockVpnService : VpnService() {
    private var vpnInterface: ParcelFileDescriptor? = null
    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())
    private lateinit var filterEngine: FilterEngine
    
    override fun onCreate() {
        super.onCreate()
        filterEngine = FilterEngine(loadFilterRules())
    }
    
    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (intent?.action == ACTION_START) {
            startVpn()
        } else if (intent?.action == ACTION_STOP) {
            stopVpn()
        }
        return START_STICKY
    }
    
    private fun startVpn() {
        val builder = Builder()
        builder.setSession("AdBlocker")
            .addAddress("10.0.0.2", 32)
            .addRoute("0.0.0.0", 0)
            .addDnsServer("8.8.8.8")
            .setMtu(1500)
        
        vpnInterface = builder.establish()
        
        scope.launch {
            handlePackets()
        }
    }
    
    private suspend fun handlePackets() = withContext(Dispatchers.IO) {
        val vpnInput = vpnInterface!!.fileDescriptor
        val buffer = ByteBuffer.allocate(32767)
        
        while (isActive) {
            val length = readPacket(vpnInput, buffer)
            if (length > 0) {
                buffer.flip()
                val packet = parsePacket(buffer)
                
                if (shouldBlockPacket(packet)) {
                    // Drop packet
                    continue
                }
                
                // Forward packet
                forwardPacket(packet)
            }
            buffer.clear()
        }
    }
    
    private fun shouldBlockPacket(packet: Packet): Boolean {
        return when (packet) {
            is DnsPacket -> filterEngine.shouldBlockDomain(packet.domain)
            is HttpPacket -> filterEngine.shouldBlockUrl(packet.url)
            else -> false
        }
    }
}
```

#### 4.2.2 YouTube広告ブロック
```kotlin
// YouTubeAdBlocker.kt
package com.adblocker.android.youtube

import android.webkit.WebView
import android.webkit.WebViewClient

class YouTubeAdBlocker {
    companion object {
        private const val YOUTUBE_AD_BLOCK_SCRIPT = """
            (function() {
                // Override fetch to block ad requests
                const originalFetch = window.fetch;
                window.fetch = function(...args) {
                    const url = args[0];
                    if (typeof url === 'string') {
                        // Block known YouTube ad endpoints
                        if (url.includes('doubleclick.net') ||
                            url.includes('/api/stats/ads') ||
                            url.includes('/pagead/') ||
                            url.includes('/ptracking')) {
                            return Promise.resolve(new Response('', { status: 200 }));
                        }
                    }
                    return originalFetch.apply(this, args);
                };
                
                // Skip video ads
                setInterval(() => {
                    const skipButton = document.querySelector('.ytp-ad-skip-button');
                    if (skipButton) {
                        skipButton.click();
                    }
                    
                    const video = document.querySelector('video');
                    if (video && document.querySelector('.ad-showing')) {
                        video.currentTime = video.duration;
                    }
                }, 100);
                
                // Remove ad overlays
                const style = document.createElement('style');
                style.textContent = `
                    .ytp-ad-overlay-container,
                    .ytp-ad-text-overlay,
                    .video-ads,
                    .ytp-ad-module {
                        display: none !important;
                    }
                `;
                document.head.appendChild(style);
            })();
        """
    }
    
    fun injectIntoWebView(webView: WebView) {
        webView.evaluateJavascript(YOUTUBE_AD_BLOCK_SCRIPT, null)
    }
}
```

### 4.3 iOS実装

#### 4.3.1 Network Extension
```swift
// PacketTunnelProvider.swift
import NetworkExtension

class PacketTunnelProvider: NEPacketTunnelProvider {
    private var filterEngine: FilterEngine!
    
    override func startTunnel(options: [String : NSObject]?, completionHandler: @escaping (Error?) -> Void) {
        let settings = NEPacketTunnelNetworkSettings(tunnelRemoteAddress: "10.0.0.1")
        
        settings.ipv4Settings = NEIPv4Settings(addresses: ["10.0.0.2"], subnetMasks: ["255.255.255.255"])
        settings.ipv4Settings?.includedRoutes = [NEIPv4Route.default()]
        
        settings.dnsSettings = NEDNSSettings(servers: ["8.8.8.8", "8.8.4.4"])
        
        setTunnelNetworkSettings(settings) { error in
            if error == nil {
                self.filterEngine = FilterEngine(rules: self.loadFilterRules())
                self.startHandlingPackets()
            }
            completionHandler(error)
        }
    }
    
    private func startHandlingPackets() {
        packetFlow.readPackets { packets, protocols in
            var packetsToForward: [Data] = []
            
            for (index, packet) in packets.enumerated() {
                let proto = protocols[index]
                
                if self.shouldBlockPacket(packet, protocol: proto) {
                    // Drop packet
                    continue
                }
                
                packetsToForward.append(packet)
            }
            
            if !packetsToForward.isEmpty {
                self.packetFlow.writePackets(packetsToForward, withProtocols: protocols)
            }
            
            // Continue reading
            self.startHandlingPackets()
        }
    }
    
    private func shouldBlockPacket(_ packet: Data, protocol proto: NSNumber) -> Bool {
        // Parse packet and check against filter rules
        if let dnsQuery = DNSPacket(data: packet) {
            return filterEngine.shouldBlockDomain(dnsQuery.domain)
        }
        
        return false
    }
}
```

#### 4.3.2 Safari Content Blocker
```swift
// ContentBlockerRequestHandler.swift
import SafariServices

class ContentBlockerRequestHandler: NSObject, NSExtensionRequestHandling {
    
    func beginRequest(with context: NSExtensionContext) {
        let rules = generateBlockingRules()
        
        let attachment = NSItemProvider(contentsOf: writeRulesToFile(rules))!
        let item = NSExtensionItem()
        item.attachments = [attachment]
        
        context.completeRequest(returningItems: [item], completionHandler: nil)
    }
    
    private func generateBlockingRules() -> [[String: Any]] {
        var rules: [[String: Any]] = []
        
        // YouTube広告ブロックルール
        rules.append([
            "trigger": [
                "url-filter": ".*doubleclick\\.net.*",
                "if-domain": ["youtube.com", "m.youtube.com"]
            ],
            "action": ["type": "block"]
        ])
        
        rules.append([
            "trigger": [
                "url-filter": ".*\\.googlevideo\\.com/videoplayback.*ctier=L.*",
                "if-domain": ["youtube.com"]
            ],
            "action": ["type": "block"]
        ])
        
        // CSS要素隠蔽ルール
        rules.append([
            "trigger": [
                "url-filter": ".*",
                "if-domain": ["youtube.com"]
            ],
            "action": [
                "type": "css-display-none",
                "selector": ".ytp-ad-overlay-container, .ytp-ad-text-overlay"
            ]
        ])
        
        return rules
    }
}
```

## 5. データフロー

### 5.1 Android VPNモード
```
アプリ → Android System → VPN Service → Filter Engine → 判定
  ↓                                                         ↓
  ←─────────────────── (Blocked) ←──────────────────────────┘
  ↓
  ←─────────────────── Internet ←─────────── (Allowed) ←────┘
```

### 5.2 iOS Network Extension
```
アプリ → iOS System → Network Extension → Filter Engine → 判定
  ↓                                                         ↓
  ←─────────────────── (Blocked) ←──────────────────────────┘
  ↓
  ←─────────────────── Internet ←─────────── (Allowed) ←────┘
```

## 6. フィルターリスト管理

### 6.1 フィルター形式
```json
{
  "version": "1.0.0",
  "rules": [
    {
      "type": "url_block",
      "pattern": "||doubleclick.net^"
    },
    {
      "type": "css_hide",
      "selector": ".ad-banner",
      "domains": ["example.com"]
    },
    {
      "type": "script_inject",
      "script": "console.log('Ad blocked');",
      "domains": ["youtube.com"]
    }
  ]
}
```

### 6.2 更新メカニズム
- 自動更新間隔: 24時間
- 差分更新対応
- 圧縮転送 (gzip)
- 署名検証によるセキュリティ確保

## 7. パフォーマンス最適化

### 7.1 メモリ最適化
- Rust側でのゼロコピー処理
- メモリマップドファイルによるルール読み込み
- 遅延初期化

### 7.2 バッテリー最適化
- Dozeモード対応（Android）
- Background App Refresh最適化（iOS）
- 効率的なパケット処理

### 7.3 起動時間短縮
```rust
// 段階的初期化
impl FilterEngine {
    pub fn new_lazy() -> Self {
        FilterEngine {
            url_matcher: Arc::new(AhoCorasick::new(&[]).unwrap()),
            css_rules: Vec::new(),
            script_rules: Vec::new(),
        }
    }
    
    pub fn load_essential_rules(&mut self) {
        // 最重要ルールのみ先行ロード
    }
    
    pub fn load_full_rules(&mut self) {
        // バックグラウンドで残りをロード
    }
}
```

## 8. テスト戦略

### 8.1 ユニットテスト
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_url_blocking() {
        let rules = vec![
            FilterRule::UrlBlock("||doubleclick.net^".to_string()),
        ];
        let engine = FilterEngine::new(&rules);
        
        assert!(engine.should_block_url("https://doubleclick.net/ad"));
        assert!(!engine.should_block_url("https://example.com"));
    }
}
```

### 8.2 統合テスト
- 実際の広告URLでのテスト
- YouTube広告ブロックの検証
- パフォーマンステスト

## 9. ビルド・配布

### 9.1 Android
```bash
# Rustライブラリビルド
cargo build --target aarch64-linux-android --release
cargo build --target armv7-linux-androideabi --release

# APKビルド
./gradlew assembleRelease

# 署名
jarsigner -keystore release.keystore app-release.apk
```

### 9.2 iOS
```bash
# Rustライブラリビルド
cargo build --target aarch64-apple-ios --release
cargo build --target x86_64-apple-ios --release

# XCFramework作成
xcodebuild -create-xcframework \
  -library target/aarch64-apple-ios/release/libadblock.a \
  -library target/x86_64-apple-ios/release/libadblock.a \
  -output AdBlockCore.xcframework
```

## 10. セキュリティ考慮事項

### 10.1 プライバシー保護
- ログ収集なし
- ユーザートラッキングなし
- 全処理をローカルで実行

### 10.2 通信セキュリティ
- フィルターリスト更新時のHTTPS必須
- 証明書ピンニング
- 署名検証

### 10.3 アプリセキュリティ
- コード難読化
- Anti-tampering対策
- Root/Jailbreak検出（オプション）