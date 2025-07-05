# システムアーキテクチャ設計書

## 1. アーキテクチャ概要

### 1.1 設計原則
- **Cross-Platform First**: 共通コードの最大化
- **Performance Oriented**: 高速・軽量を最優先
- **Privacy by Design**: ユーザーデータの完全ローカル処理
- **Modular Architecture**: 機能単位での独立性確保

### 1.2 レイヤー構成

```
┌──────────────────────────────────────────────────────────┐
│                    Presentation Layer                     │
├──────────────────────┬───────────────────────────────────┤
│     Android UI       │            iOS UI                 │
│   (Jetpack Compose)  │          (SwiftUI)               │
├──────────────────────┴───────────────────────────────────┤
│                    Application Layer                      │
├──────────────────────┬───────────────────────────────────┤
│   Android Service    │         iOS Extension            │
│   (VPN Service)      │    (Network Extension)           │
├──────────────────────┴───────────────────────────────────┤
│                     Domain Layer                          │
│                  (Business Logic)                         │
├──────────────────────────────────────────────────────────┤
│                  Infrastructure Layer                     │
│                    (Rust Core)                           │
└──────────────────────────────────────────────────────────┘
```

## 2. コンポーネント設計

### 2.1 Core Components (Rust)

#### 2.1.1 Filter Engine
```rust
pub mod filter_engine {
    pub struct FilterEngine {
        url_matcher: UrlMatcher,
        css_injector: CssInjector,
        script_injector: ScriptInjector,
        rule_manager: RuleManager,
    }
    
    pub trait FilterStrategy {
        fn should_block(&self, request: &Request) -> BlockDecision;
        fn inject_rules(&self, response: &mut Response);
    }
}
```

#### 2.1.2 Network Interceptor
```rust
pub mod network {
    pub struct NetworkInterceptor {
        packet_parser: PacketParser,
        dns_resolver: DnsResolver,
        http_handler: HttpHandler,
        https_handler: HttpsHandler,
    }
    
    pub enum InterceptMode {
        VpnMode,        // Android
        NetworkExtMode, // iOS
        ProxyMode,      // Fallback
    }
}
```

#### 2.1.3 Rule Manager
```rust
pub mod rules {
    pub struct RuleManager {
        local_storage: LocalRuleStorage,
        remote_updater: RemoteRuleUpdater,
        rule_compiler: RuleCompiler,
    }
    
    pub struct CompiledRuleset {
        url_patterns: AhoCorasick,
        css_rules: HashMap<String, Vec<CssRule>>,
        script_rules: HashMap<String, Vec<ScriptRule>>,
        metadata: RulesetMetadata,
    }
}
```

### 2.2 Platform Bridges

#### 2.2.1 Android Bridge (JNI)
```kotlin
// Kotlin側
class AdBlockCore {
    companion object {
        init {
            System.loadLibrary("adblock_core")
        }
    }
    
    external fun createEngine(configJson: String): Long
    external fun shouldBlock(enginePtr: Long, url: String): Boolean
    external fun destroyEngine(enginePtr: Long)
    
    // High-level wrapper
    class Engine(config: EngineConfig) : AutoCloseable {
        private val ptr = createEngine(config.toJson())
        
        fun shouldBlock(url: String) = shouldBlock(ptr, url)
        
        override fun close() = destroyEngine(ptr)
    }
}
```

```rust
// Rust側 JNI実装
#[no_mangle]
pub extern "system" fn Java_com_adblock_core_AdBlockCore_createEngine(
    env: JNIEnv,
    _class: JClass,
    config_json: JString,
) -> jlong {
    let config_str: String = env.get_string(config_json).unwrap().into();
    let engine = Box::new(FilterEngine::new_from_json(&config_str));
    Box::into_raw(engine) as jlong
}
```

#### 2.2.2 iOS Bridge (Swift/C)
```swift
// Swift側
class AdBlockCore {
    private let engine: OpaquePointer
    
    init(config: EngineConfig) throws {
        guard let configData = try? JSONEncoder().encode(config),
              let configString = String(data: configData, encoding: .utf8) else {
            throw AdBlockError.invalidConfig
        }
        
        engine = adblock_create_engine(configString)
    }
    
    func shouldBlock(url: String) -> Bool {
        return adblock_should_block(engine, url)
    }
    
    deinit {
        adblock_destroy_engine(engine)
    }
}
```

### 2.3 データフロー設計

#### 2.3.1 Android VPN Mode
```
┌─────────────┐     ┌──────────────┐     ┌──────────────┐
│   App A     │────▶│ Android VPN  │────▶│ Filter Core  │
└─────────────┘     │   Service    │     └──────┬───────┘
┌─────────────┐     │              │            │
│   App B     │────▶│  ┌────────┐  │     ┌──────▼───────┐
└─────────────┘     │  │ Packet │  │     │   Decision   │
┌─────────────┐     │  │ Parser │  │     │   Engine     │
│   Browser   │────▶│  └────────┘  │     └──────┬───────┘
└─────────────┘     └──────────────┘            │
                            ▲                    ▼
                            └────────────────────┘
                              Block or Forward
```

#### 2.3.2 iOS Network Extension Mode
```
┌─────────────┐     ┌──────────────┐     ┌──────────────┐
│   Safari    │────▶│   Network    │────▶│ Filter Core  │
└─────────────┘     │  Extension   │     └──────┬───────┘
┌─────────────┐     │              │            │
│   App A     │────▶│ ┌──────────┐ │     ┌──────▼───────┐
└─────────────┘     │ │  Packet  │ │     │  Rule Match  │
┌─────────────┐     │ │   Flow   │ │     │   Engine     │
│   App B     │────▶│ └──────────┘ │     └──────┬───────┘
└─────────────┘     └──────────────┘            │
                            ▲                    ▼
                            └────────────────────┘
```

## 3. モジュール間インターフェース

### 3.1 Core API (Rust)
```rust
// Public API
pub mod api {
    use crate::{FilterEngine, Request, Response, BlockDecision};
    
    #[repr(C)]
    pub struct CFilterEngine {
        inner: *mut FilterEngine,
    }
    
    #[no_mangle]
    pub extern "C" fn filter_engine_create(
        config: *const c_char
    ) -> *mut CFilterEngine {
        // Implementation
    }
    
    #[no_mangle]
    pub extern "C" fn filter_engine_process_request(
        engine: *const CFilterEngine,
        request: *const c_char,
    ) -> BlockDecision {
        // Implementation
    }
    
    #[no_mangle]
    pub extern "C" fn filter_engine_destroy(
        engine: *mut CFilterEngine
    ) {
        // Implementation
    }
}
```

### 3.2 Platform Service Interface

#### Android
```kotlin
interface IAdBlockService {
    fun startBlocking(): Result<Unit>
    fun stopBlocking(): Result<Unit>
    fun getStatistics(): BlockingStatistics
    fun updateRules(): Result<Unit>
    fun setConfiguration(config: ServiceConfig): Result<Unit>
}

class AdBlockVpnService : VpnService(), IAdBlockService {
    private val engine by lazy { AdBlockCore.Engine(loadConfig()) }
    
    override fun startBlocking(): Result<Unit> {
        return try {
            establishVpn()
            startPacketProcessing()
            Result.success(Unit)
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
}
```

#### iOS
```swift
protocol AdBlockServiceProtocol {
    func startBlocking() async throws
    func stopBlocking() async throws
    func getStatistics() async -> BlockingStatistics
    func updateRules() async throws
    func setConfiguration(_ config: ServiceConfig) async throws
}

class AdBlockNetworkExtension: NEPacketTunnelProvider, AdBlockServiceProtocol {
    private let engine = AdBlockCore(config: loadConfig())
    
    func startBlocking() async throws {
        let settings = createTunnelSettings()
        try await setTunnelNetworkSettings(settings)
        startPacketHandling()
    }
}
```

## 4. 状態管理

### 4.1 グローバル状態
```rust
pub struct GlobalState {
    pub blocking_enabled: AtomicBool,
    pub statistics: Arc<Mutex<Statistics>>,
    pub configuration: Arc<RwLock<Configuration>>,
    pub rule_version: AtomicU32,
}

pub struct Statistics {
    pub blocked_requests: u64,
    pub allowed_requests: u64,
    pub data_saved: u64,
    pub blocking_history: VecDeque<BlockEvent>,
}
```

### 4.2 プラットフォーム固有状態

#### Android
```kotlin
@Entity
data class BlockingState(
    @PrimaryKey val id: Int = 0,
    val isEnabled: Boolean,
    val lastUpdated: Long,
    val totalBlocked: Long,
    val totalAllowed: Long
)

class StateRepository(private val dao: StateDao) {
    fun getState() = dao.getState().asLiveData()
    suspend fun updateState(state: BlockingState) = dao.update(state)
}
```

#### iOS
```swift
@Observable
class BlockingState {
    @AppStorage("isBlockingEnabled") var isEnabled = false
    @AppStorage("lastRuleUpdate") var lastRuleUpdate = Date()
    
    var statistics = Statistics()
    var configuration = Configuration()
}
```

## 5. エラーハンドリング

### 5.1 エラー階層
```rust
#[derive(Debug, thiserror::Error)]
pub enum AdBlockError {
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),
    
    #[error("Filter error: {0}")]
    Filter(#[from] FilterError),
    
    #[error("Configuration error: {0}")]
    Configuration(#[from] ConfigError),
    
    #[error("Platform error: {0}")]
    Platform(String),
}

pub type Result<T> = std::result::Result<T, AdBlockError>;
```

### 5.2 エラー伝播
```kotlin
// Android
sealed class AdBlockResult<out T> {
    data class Success<T>(val data: T) : AdBlockResult<T>()
    data class Error(val error: AdBlockError) : AdBlockResult<Nothing>()
}

// iOS
enum AdBlockError: Error {
    case network(NetworkError)
    case filter(FilterError)
    case configuration(ConfigError)
    case platform(String)
}
```

## 6. パフォーマンス最適化アーキテクチャ

### 6.1 メモリ管理
```rust
pub struct MemoryPool {
    packet_buffers: Vec<ByteBuffer>,
    string_cache: StringInterner,
    rule_cache: LruCache<RuleId, CompiledRule>,
}

impl MemoryPool {
    pub fn acquire_buffer(&mut self) -> ByteBuffer {
        self.packet_buffers.pop()
            .unwrap_or_else(|| ByteBuffer::with_capacity(1500))
    }
    
    pub fn release_buffer(&mut self, buffer: ByteBuffer) {
        if self.packet_buffers.len() < 100 {
            self.packet_buffers.push(buffer);
        }
    }
}
```

### 6.2 並行処理設計
```rust
pub struct ConcurrentProcessor {
    workers: Vec<Worker>,
    task_queue: Arc<SegQueue<Task>>,
    result_aggregator: Arc<Mutex<ResultAggregator>>,
}

impl ConcurrentProcessor {
    pub fn process_packet(&self, packet: Packet) {
        let task = Task::ProcessPacket(packet);
        self.task_queue.push(task);
    }
}
```

## 7. セキュリティアーキテクチャ

### 7.1 暗号化レイヤー
```rust
pub struct SecurityLayer {
    config_encryptor: AesGcm<Aes256>,
    rule_verifier: Ed25519Verifier,
    cert_validator: CertificateValidator,
}
```

### 7.2 サンドボックス設計
- Rust Core: メモリ安全性保証
- Android: SELinux ポリシー適用
- iOS: App Sandbox + Network Extension制限

## 8. 拡張性設計

### 8.1 プラグインアーキテクチャ
```rust
pub trait FilterPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> Version;
    fn process(&self, request: &Request) -> PluginResult;
}

pub struct PluginManager {
    plugins: Vec<Box<dyn FilterPlugin>>,
}
```

### 8.2 将来の拡張ポイント
1. **新プロトコル対応**: QUIC, HTTP/3
2. **AI/ML統合**: 動的広告検出
3. **クラウド統合**: 設定同期（オプション）
4. **新プラットフォーム**: macOS, Windows

## 9. デプロイメントアーキテクチャ

### 9.1 ビルドパイプライン
```yaml
# CI/CD Pipeline
stages:
  - rust-core:
      - cargo test
      - cargo build --release
      - generate bindings
  
  - android:
      - gradle test
      - gradle assembleRelease
      - sign APK
  
  - ios:
      - xcodebuild test
      - xcodebuild archive
      - notarize
```

### 9.2 配布戦略
- Android: GitHub Releases + F-Droid
- iOS: TestFlight + サイドロード向けIPA