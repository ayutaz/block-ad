# 実装ガイドライン

## 1. 開発環境セットアップ

### 1.1 必要なツール
```bash
# Rust (共通コア)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add aarch64-linux-android armv7-linux-androideabi
rustup target add aarch64-apple-ios x86_64-apple-ios

# Android開発
# Android Studio (https://developer.android.com/studio)
# NDK r25c以上
# CMake 3.18以上

# iOS開発  
# Xcode 14.0以上
# CocoaPods or Swift Package Manager
# Apple Developer Account (Network Extension用)

# 共通ツール
cargo install cargo-ndk
cargo install cbindgen  # ヘッダー生成用
```

### 1.2 プロジェクト構造
```
block-ad/
├── core/                    # Rustコア
│   ├── src/
│   │   ├── lib.rs
│   │   ├── filter_engine.rs
│   │   ├── network.rs
│   │   ├── ffi/           # FFIバインディング
│   │   └── rules/
│   ├── Cargo.toml
│   └── build.rs
├── android/                 # Androidアプリ
│   ├── app/
│   │   ├── src/main/
│   │   │   ├── java/
│   │   │   ├── kotlin/
│   │   │   ├── cpp/      # JNIブリッジ
│   │   │   └── res/
│   │   └── build.gradle.kts
│   └── settings.gradle.kts
├── ios/                     # iOSアプリ
│   ├── AdBlocker/
│   │   ├── Core/          # Swiftラッパー
│   │   ├── UI/
│   │   └── Extensions/
│   ├── AdBlocker.xcodeproj
│   └── Podfile
├── shared/                  # 共通リソース
│   ├── rules/              # フィルタールール
│   └── assets/
└── scripts/                 # ビルドスクリプト
```

## 2. 実装手順

### 2.1 Phase 1: Rustコア開発（1-2週間）

#### Step 1: 基本構造の実装
```rust
// core/src/lib.rs
#![allow(non_snake_case)]

pub mod filter_engine;
pub mod network;
pub mod ffi;
pub mod rules;

use filter_engine::FilterEngine;
use std::sync::Arc;

pub struct AdBlockCore {
    engine: Arc<FilterEngine>,
}

impl AdBlockCore {
    pub fn new(config: Config) -> Result<Self> {
        let engine = FilterEngine::new(config)?;
        Ok(Self {
            engine: Arc::new(engine),
        })
    }
}

// core/src/filter_engine.rs
use aho_corasick::AhoCorasick;

pub struct FilterEngine {
    url_matcher: AhoCorasick,
    domain_rules: HashMap<String, Vec<Rule>>,
    css_rules: Vec<CssRule>,
}

impl FilterEngine {
    pub fn new(config: Config) -> Result<Self> {
        // 実装
    }
    
    pub fn should_block(&self, request: &Request) -> BlockDecision {
        // URLマッチング
        if self.url_matcher.is_match(&request.url) {
            return BlockDecision::block("URL pattern matched");
        }
        
        // ドメインルールチェック
        if let Some(domain) = extract_domain(&request.url) {
            if let Some(rules) = self.domain_rules.get(domain) {
                // ルール評価
            }
        }
        
        BlockDecision::allow()
    }
}
```

#### Step 2: FFIバインディング作成
```rust
// core/src/ffi/mod.rs
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn adblock_create(config_json: *const c_char) -> *mut AdBlockCore {
    let config_str = unsafe {
        CStr::from_ptr(config_json).to_string_lossy()
    };
    
    match serde_json::from_str(&config_str) {
        Ok(config) => {
            match AdBlockCore::new(config) {
                Ok(core) => Box::into_raw(Box::new(core)),
                Err(_) => std::ptr::null_mut(),
            }
        }
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn adblock_should_block(
    core: *const AdBlockCore,
    url: *const c_char,
) -> bool {
    if core.is_null() || url.is_null() {
        return false;
    }
    
    let core = unsafe { &*core };
    let url = unsafe { CStr::from_ptr(url).to_string_lossy() };
    
    let request = Request::new(url.to_string());
    core.engine.should_block(&request).should_block
}
```

#### Step 3: ビルド設定
```toml
# core/Cargo.toml
[package]
name = "adblock-core"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "staticlib"]

[dependencies]
aho-corasick = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
regex = "1.9"
once_cell = "1.18"

[target.'cfg(target_os = "android")'.dependencies]
jni = "0.21"
android_logger = "0.13"

[target.'cfg(target_os = "ios")'.dependencies]
objc = "0.2"
```

### 2.2 Phase 2: Android実装（2-3週間）

#### Step 1: JNIブリッジ実装
```kotlin
// android/app/src/main/kotlin/com/adblock/core/NativeLib.kt
package com.adblock.core

object NativeLib {
    init {
        System.loadLibrary("adblock_core")
    }
    
    external fun createEngine(configJson: String): Long
    external fun shouldBlock(enginePtr: Long, url: String): Boolean
    external fun destroyEngine(enginePtr: Long)
}

// Kotlinラッパー
class AdBlockEngine(config: EngineConfig) : Closeable {
    private val ptr: Long = NativeLib.createEngine(config.toJson())
    
    init {
        require(ptr != 0L) { "Failed to create engine" }
    }
    
    fun shouldBlock(url: String): Boolean {
        return NativeLib.shouldBlock(ptr, url)
    }
    
    override fun close() {
        NativeLib.destroyEngine(ptr)
    }
}
```

#### Step 2: VPN Service実装
```kotlin
// android/app/src/main/kotlin/com/adblock/service/AdBlockVpnService.kt
class AdBlockVpnService : VpnService() {
    private lateinit var engine: AdBlockEngine
    private var vpnInterface: ParcelFileDescriptor? = null
    
    override fun onCreate() {
        super.onCreate()
        engine = AdBlockEngine(loadConfig())
        startForeground(NOTIFICATION_ID, createNotification())
    }
    
    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        when (intent?.action) {
            ACTION_START -> startVpn()
            ACTION_STOP -> stopVpn()
        }
        return START_STICKY
    }
    
    private fun startVpn() {
        val builder = Builder()
            .setSession("AdBlocker")
            .addAddress("10.0.0.2", 32)
            .addRoute("0.0.0.0", 0)
            .addDnsServer("8.8.8.8")
            .setMtu(1500)
            
        // アプリ除外設定
        getExcludedApps().forEach { 
            builder.addDisallowedApplication(it)
        }
        
        vpnInterface = builder.establish()
        startPacketProcessing()
    }
    
    private fun startPacketProcessing() {
        thread {
            val vpnInput = FileInputStream(vpnInterface!!.fileDescriptor)
            val vpnOutput = FileOutputStream(vpnInterface!!.fileDescriptor)
            val buffer = ByteBuffer.allocate(32767)
            
            while (true) {
                val length = vpnInput.channel.read(buffer)
                if (length > 0) {
                    buffer.flip()
                    processPacket(buffer, vpnOutput)
                    buffer.clear()
                }
            }
        }
    }
}
```

#### Step 3: UI実装
```kotlin
// Jetpack Compose UI
@Composable
fun MainScreen(viewModel: MainViewModel) {
    val uiState by viewModel.uiState.collectAsState()
    
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("AdBlocker") },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = MaterialTheme.colorScheme.primary
                )
            )
        }
    ) { paddingValues ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues)
                .padding(16.dp)
        ) {
            // メイントグル
            Card(
                modifier = Modifier.fillMaxWidth(),
                elevation = CardDefaults.cardElevation(8.dp)
            ) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(16.dp),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text(
                        "広告ブロック",
                        style = MaterialTheme.typography.headlineSmall
                    )
                    Switch(
                        checked = uiState.isEnabled,
                        onCheckedChange = { viewModel.toggleBlocking() }
                    )
                }
            }
            
            Spacer(modifier = Modifier.height(16.dp))
            
            // 統計表示
            StatisticsCard(statistics = uiState.statistics)
        }
    }
}
```

### 2.3 Phase 3: iOS実装（2-3週間）

#### Step 1: Swiftブリッジ実装
```swift
// ios/AdBlocker/Core/AdBlockCore.swift
import Foundation

class AdBlockCore {
    private let engine: OpaquePointer
    
    init(config: EngineConfig) throws {
        let configData = try JSONEncoder().encode(config)
        guard let configString = String(data: configData, encoding: .utf8) else {
            throw AdBlockError.invalidConfiguration
        }
        
        guard let engine = adblock_create(configString) else {
            throw AdBlockError.engineCreationFailed
        }
        
        self.engine = engine
    }
    
    func shouldBlock(url: String) -> Bool {
        return adblock_should_block(engine, url)
    }
    
    deinit {
        adblock_destroy(engine)
    }
}

// C関数の宣言
@_silgen_name("adblock_create")
func adblock_create(_ config: UnsafePointer<CChar>) -> OpaquePointer?

@_silgen_name("adblock_should_block")
func adblock_should_block(_ engine: OpaquePointer, _ url: UnsafePointer<CChar>) -> Bool

@_silgen_name("adblock_destroy")
func adblock_destroy(_ engine: OpaquePointer)
```

#### Step 2: Network Extension実装
```swift
// ios/AdBlockerNetworkExtension/PacketTunnelProvider.swift
import NetworkExtension

class PacketTunnelProvider: NEPacketTunnelProvider {
    private var core: AdBlockCore?
    
    override func startTunnel(options: [String : NSObject]?, completionHandler: @escaping (Error?) -> Void) {
        NEProvider.startSystemExtensionMode()
        
        do {
            // コア初期化
            let config = try loadConfiguration()
            self.core = try AdBlockCore(config: config)
            
            // トンネル設定
            let settings = NEPacketTunnelNetworkSettings(tunnelRemoteAddress: "10.0.0.1")
            settings.ipv4Settings = NEIPv4Settings(
                addresses: ["10.0.0.2"],
                subnetMasks: ["255.255.255.255"]
            )
            settings.ipv4Settings?.includedRoutes = [NEIPv4Route.default()]
            settings.dnsSettings = NEDNSSettings(servers: ["8.8.8.8"])
            
            setTunnelNetworkSettings(settings) { error in
                if let error = error {
                    completionHandler(error)
                } else {
                    self.startHandlingPackets()
                    completionHandler(nil)
                }
            }
        } catch {
            completionHandler(error)
        }
    }
    
    private func startHandlingPackets() {
        packetFlow.readPackets { [weak self] packets, protocols in
            guard let self = self else { return }
            
            let filteredPackets = packets.enumerated().compactMap { index, packet -> Data? in
                if self.shouldBlockPacket(packet) {
                    // ブロック
                    return nil
                }
                return packet
            }
            
            if !filteredPackets.isEmpty {
                self.packetFlow.writePackets(filteredPackets, withProtocols: protocols)
            }
            
            self.startHandlingPackets() // 継続
        }
    }
}
```

#### Step 3: SwiftUI実装
```swift
// ios/AdBlocker/UI/ContentView.swift
import SwiftUI

struct ContentView: View {
    @StateObject private var viewModel = MainViewModel()
    
    var body: some View {
        NavigationView {
            VStack(spacing: 20) {
                // メイントグル
                VStack(alignment: .leading, spacing: 12) {
                    HStack {
                        Text("広告ブロック")
                            .font(.title2)
                            .fontWeight(.semibold)
                        
                        Spacer()
                        
                        Toggle("", isOn: $viewModel.isBlockingEnabled)
                            .labelsHidden()
                            .toggleStyle(SwitchToggleStyle(tint: .blue))
                    }
                    
                    Text(viewModel.isBlockingEnabled ? "有効" : "無効")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                .padding()
                .background(Color(.systemGray6))
                .cornerRadius(12)
                
                // 統計
                StatisticsView(statistics: viewModel.statistics)
                
                Spacer()
            }
            .padding()
            .navigationTitle("AdBlocker")
        }
    }
}
```

## 3. テスト実装

### 3.1 ユニットテスト
```rust
// core/src/filter_engine.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_url_blocking() {
        let config = Config {
            rules: vec![
                Rule::url_pattern("||doubleclick.net^"),
                Rule::url_pattern("||googleadservices.com^"),
            ],
        };
        
        let engine = FilterEngine::new(config).unwrap();
        
        assert!(engine.should_block(&Request {
            url: "https://doubleclick.net/ad".to_string(),
            ..Default::default()
        }).should_block);
        
        assert!(!engine.should_block(&Request {
            url: "https://example.com".to_string(),
            ..Default::default()
        }).should_block);
    }
    
    #[test]
    fn test_youtube_ad_blocking() {
        let config = Config::with_youtube_rules();
        let engine = FilterEngine::new(config).unwrap();
        
        let youtube_ad_urls = vec![
            "https://www.youtube.com/api/stats/ads",
            "https://www.youtube.com/pagead/",
            "https://doubleclick.net/some/path",
        ];
        
        for url in youtube_ad_urls {
            assert!(engine.should_block(&Request {
                url: url.to_string(),
                ..Default::default()
            }).should_block);
        }
    }
}
```

### 3.2 統合テスト
```kotlin
// Android統合テスト
@Test
fun testVpnServiceIntegration() {
    val scenario = ActivityScenario.launch(MainActivity::class.java)
    
    scenario.onActivity { activity ->
        // VPNサービス開始
        val intent = Intent(activity, AdBlockVpnService::class.java)
            .setAction(AdBlockVpnService.ACTION_START)
        activity.startService(intent)
        
        // 状態確認
        Thread.sleep(1000)
        assertTrue(AdBlockVpnService.isRunning)
        
        // ブロックテスト
        val testUrl = "https://doubleclick.net/test"
        val blocked = activity.viewModel.shouldBlock(testUrl)
        assertTrue(blocked)
    }
}
```

## 4. パフォーマンス最適化

### 4.1 Rust最適化
```rust
// リリースビルド最適化
// Cargo.toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true

// メモリプール実装
pub struct PacketPool {
    pool: Vec<Box<[u8; PACKET_SIZE]>>,
}

impl PacketPool {
    pub fn acquire(&mut self) -> Box<[u8; PACKET_SIZE]> {
        self.pool.pop().unwrap_or_else(|| {
            Box::new([0u8; PACKET_SIZE])
        })
    }
    
    pub fn release(&mut self, packet: Box<[u8; PACKET_SIZE]>) {
        if self.pool.len() < MAX_POOL_SIZE {
            self.pool.push(packet);
        }
    }
}
```

### 4.2 プラットフォーム最適化
```kotlin
// Android: バッテリー最適化
class BatteryOptimizedScheduler {
    fun scheduleRuleUpdate(context: Context) {
        val constraints = Constraints.Builder()
            .setRequiredNetworkType(NetworkType.UNMETERED)
            .setRequiresBatteryNotLow(true)
            .setRequiresCharging(false)
            .build()
            
        val updateWork = PeriodicWorkRequestBuilder<RuleUpdateWorker>(
            24, TimeUnit.HOURS
        )
            .setConstraints(constraints)
            .build()
            
        WorkManager.getInstance(context)
            .enqueueUniquePeriodicWork(
                "rule_update",
                ExistingPeriodicWorkPolicy.KEEP,
                updateWork
            )
    }
}
```

## 5. デバッグとトラブルシューティング

### 5.1 ログ設定
```rust
// Rust側ログ
#[cfg(target_os = "android")]
fn init_logging() {
    android_logger::init_once(
        android_logger::Config::default()
            .with_min_level(log::Level::Debug)
            .with_tag("AdBlockCore")
    );
}

#[cfg(target_os = "ios")]
fn init_logging() {
    env_logger::Builder::from_default_env()
        .target(env_logger::Target::Stdout)
        .init();
}
```

### 5.2 よくある問題と解決策

#### 問題1: VPNが確立できない
```kotlin
// 解決策: 権限チェック
private fun checkVpnPermission(): Boolean {
    val intent = VpnService.prepare(this)
    return intent == null
}

// MainActivityで権限リクエスト
private val vpnPermissionLauncher = registerForActivityResult(
    ActivityResultContracts.StartActivityForResult()
) { result ->
    if (result.resultCode == RESULT_OK) {
        startVpnService()
    }
}
```

#### 問題2: メモリリーク
```swift
// 解決策: weak参照使用
class PacketProcessor {
    weak var delegate: PacketProcessorDelegate?
    
    func process(_ packet: Data) {
        autoreleasepool {
            // 処理
            delegate?.didProcessPacket(packet)
        }
    }
}
```

## 6. リリース準備

### 6.1 ビルドスクリプト
```bash
#!/bin/bash
# scripts/build_all.sh

# Rust core build
echo "Building Rust core..."
cd core
cargo build --release --target aarch64-linux-android
cargo build --release --target aarch64-apple-ios
cd ..

# Android build
echo "Building Android app..."
cd android
./gradlew assembleRelease
cd ..

# iOS build
echo "Building iOS app..."
cd ios
xcodebuild -workspace AdBlocker.xcworkspace \
           -scheme AdBlocker \
           -configuration Release \
           -archivePath build/AdBlocker.xcarchive \
           archive
cd ..

echo "Build complete!"
```

### 6.2 署名とパッケージング
```kotlin
// Android署名設定
android {
    signingConfigs {
        create("release") {
            storeFile = file(System.getenv("KEYSTORE_FILE") ?: "release.keystore")
            storePassword = System.getenv("KEYSTORE_PASSWORD") ?: ""
            keyAlias = System.getenv("KEY_ALIAS") ?: ""
            keyPassword = System.getenv("KEY_PASSWORD") ?: ""
        }
    }
    
    buildTypes {
        release {
            isMinifyEnabled = true
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"))
            signingConfig = signingConfigs.getByName("release")
        }
    }
}
```

## 7. 継続的インテグレーション

### 7.1 GitHub Actions設定
```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  test-rust:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cd core && cargo test
      
  build-android:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-java@v3
        with:
          java-version: '17'
      - run: cd android && ./gradlew build
      
  build-ios:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v3
      - run: cd ios && xcodebuild test
```

この実装ガイドラインに従って開発を進めることで、高品質な広告ブロックアプリを効率的に実装できます。