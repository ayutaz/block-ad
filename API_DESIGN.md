# API・インターフェース設計書

## 1. Core Engine API (Rust)

### 1.1 基本型定義
```rust
// Types
#[repr(C)]
#[derive(Debug, Clone)]
pub struct Request {
    pub id: u64,
    pub method: RequestMethod,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub source_app: Option<String>,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub enum RequestMethod {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Connect,
    Trace,
    Patch,
}

#[repr(C)]
#[derive(Debug)]
pub struct BlockDecision {
    pub should_block: bool,
    pub reason: Option<String>,
    pub matched_rule: Option<String>,
    pub confidence: f32,
}

#[repr(C)]
#[derive(Debug)]
pub struct InjectionRules {
    pub css_rules: Vec<String>,
    pub script_rules: Vec<String>,
    pub remove_elements: Vec<String>,
}
```

### 1.2 Filter Engine API
```rust
// C-compatible API for FFI
#[no_mangle]
pub extern "C" fn adblock_engine_create(
    config_json: *const c_char
) -> *mut FilterEngine {
    // Implementation
}

#[no_mangle]
pub extern "C" fn adblock_engine_should_block(
    engine: *const FilterEngine,
    url: *const c_char,
    source_app: *const c_char,
) -> BlockDecision {
    // Implementation
}

#[no_mangle]
pub extern "C" fn adblock_engine_get_injection_rules(
    engine: *const FilterEngine,
    url: *const c_char,
) -> *mut InjectionRules {
    // Implementation
}

#[no_mangle]
pub extern "C" fn adblock_engine_update_rules(
    engine: *mut FilterEngine,
    rules_json: *const c_char,
) -> bool {
    // Implementation
}

#[no_mangle]
pub extern "C" fn adblock_engine_get_statistics(
    engine: *const FilterEngine,
) -> *mut Statistics {
    // Implementation
}

#[no_mangle]
pub extern "C" fn adblock_engine_destroy(
    engine: *mut FilterEngine
) {
    // Implementation
}
```

### 1.3 Network Interceptor API
```rust
#[no_mangle]
pub extern "C" fn adblock_packet_parse(
    packet_data: *const u8,
    packet_len: usize,
) -> *mut ParsedPacket {
    // Implementation
}

#[no_mangle]
pub extern "C" fn adblock_packet_should_block(
    engine: *const FilterEngine,
    packet: *const ParsedPacket,
) -> bool {
    // Implementation
}

#[no_mangle]
pub extern "C" fn adblock_packet_modify(
    packet: *mut ParsedPacket,
    rules: *const InjectionRules,
) -> bool {
    // Implementation
}
```

## 2. Android Platform API

### 2.1 Kotlin Service Interface
```kotlin
interface AdBlockService {
    // Lifecycle
    fun initialize(config: ServiceConfig): Result<Unit>
    fun start(): Result<Unit>
    fun stop(): Result<Unit>
    fun destroy()
    
    // Configuration
    fun updateConfiguration(config: ServiceConfig): Result<Unit>
    fun getConfiguration(): ServiceConfig
    
    // Rules
    fun updateRules(source: RuleSource): Result<RuleUpdateResult>
    fun getRuleVersion(): String
    
    // Statistics
    fun getStatistics(): Statistics
    fun resetStatistics()
    
    // Real-time monitoring
    fun observeBlockEvents(): Flow<BlockEvent>
}

data class ServiceConfig(
    val blockingEnabled: Boolean = true,
    val youtubeAdBlockEnabled: Boolean = true,
    val updateInterval: Duration = Duration.ofHours(24),
    val allowedApps: List<String> = emptyList(),
    val customRules: List<CustomRule> = emptyList(),
)

data class BlockEvent(
    val timestamp: Instant,
    val url: String,
    val sourceApp: String?,
    val rule: String,
    val blocked: Boolean,
)

data class Statistics(
    val totalBlocked: Long,
    val totalAllowed: Long,
    val dataSaved: Long,
    val topBlockedDomains: List<DomainStats>,
    val blockingHistory: List<HistoryEntry>,
)
```

### 2.2 VPN Service Implementation
```kotlin
class AdBlockVpnService : VpnService() {
    private lateinit var core: AdBlockCore
    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())
    
    // Public API
    fun configureVpn(config: VpnConfig): Result<Unit> {
        return try {
            val builder = Builder()
                .setSession("AdBlocker")
                .addAddress(config.virtualAddress, 32)
                .addRoute("0.0.0.0", 0)
                .setMtu(config.mtu)
            
            config.dnsServers.forEach { builder.addDnsServer(it) }
            vpnInterface = builder.establish()
            Result.success(Unit)
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
    
    // Packet handling
    private fun handlePacket(packet: ByteArray): PacketAction {
        val parsed = core.parsePacket(packet)
        val decision = core.shouldBlock(parsed)
        
        return when {
            decision.shouldBlock -> PacketAction.Drop
            decision.needsModification -> PacketAction.Modify(
                core.modifyPacket(parsed, decision.modifications)
            )
            else -> PacketAction.Forward
        }
    }
}

sealed class PacketAction {
    object Drop : PacketAction()
    object Forward : PacketAction()
    data class Modify(val packet: ByteArray) : PacketAction()
}
```

### 2.3 UI Components Interface
```kotlin
// ViewModel
class AdBlockViewModel(
    private val service: AdBlockService,
    private val prefs: AdBlockPreferences,
) : ViewModel() {
    
    val uiState: StateFlow<UiState> = combine(
        service.observeBlockEvents(),
        prefs.observeSettings(),
    ) { events, settings ->
        UiState(
            isEnabled = settings.blockingEnabled,
            statistics = service.getStatistics(),
            recentEvents = events.takeLast(100),
        )
    }.stateIn(viewModelScope, SharingStarted.Lazily, UiState())
    
    fun toggleBlocking() {
        viewModelScope.launch {
            val current = prefs.isBlockingEnabled()
            prefs.setBlockingEnabled(!current)
            if (!current) {
                service.start()
            } else {
                service.stop()
            }
        }
    }
}

// Composable UI
@Composable
fun AdBlockScreen(viewModel: AdBlockViewModel) {
    val uiState by viewModel.uiState.collectAsState()
    
    AdBlockTheme {
        Scaffold(
            topBar = { AdBlockTopBar() },
        ) { padding ->
            Column(modifier = Modifier.padding(padding)) {
                // Main toggle
                BlockingToggle(
                    enabled = uiState.isEnabled,
                    onToggle = { viewModel.toggleBlocking() }
                )
                
                // Statistics
                StatisticsCard(stats = uiState.statistics)
                
                // Recent activity
                RecentActivityList(events = uiState.recentEvents)
            }
        }
    }
}
```

## 3. iOS Platform API

### 3.1 Swift Service Protocol
```swift
protocol AdBlockServiceProtocol {
    // Lifecycle
    func initialize(config: ServiceConfig) async throws
    func start() async throws
    func stop() async throws
    
    // Configuration
    func updateConfiguration(_ config: ServiceConfig) async throws
    func getConfiguration() -> ServiceConfig
    
    // Rules
    func updateRules(from source: RuleSource) async throws -> RuleUpdateResult
    func getRuleVersion() -> String
    
    // Statistics
    func getStatistics() -> Statistics
    func resetStatistics() async
    
    // Real-time monitoring
    var blockEvents: AsyncStream<BlockEvent> { get }
}

struct ServiceConfig: Codable {
    var blockingEnabled: Bool = true
    var youtubeAdBlockEnabled: Bool = true
    var updateInterval: TimeInterval = 86400 // 24 hours
    var allowedApps: [String] = []
    var customRules: [CustomRule] = []
}

struct BlockEvent: Codable {
    let timestamp: Date
    let url: String
    let sourceApp: String?
    let rule: String
    let blocked: Bool
}

struct Statistics: Codable {
    let totalBlocked: Int64
    let totalAllowed: Int64
    let dataSaved: Int64
    let topBlockedDomains: [DomainStats]
    let blockingHistory: [HistoryEntry]
}
```

### 3.2 Network Extension Implementation
```swift
class PacketTunnelProvider: NEPacketTunnelProvider {
    private var core: AdBlockCore!
    private let queue = DispatchQueue(label: "adblock.packet", attributes: .concurrent)
    
    override func startTunnel(
        options: [String : NSObject]?,
        completionHandler: @escaping (Error?) -> Void
    ) {
        Task {
            do {
                // Initialize core
                let config = loadConfiguration()
                self.core = try AdBlockCore(config: config)
                
                // Configure tunnel
                let settings = createTunnelSettings()
                try await setTunnelNetworkSettings(settings)
                
                // Start packet processing
                startPacketProcessing()
                
                completionHandler(nil)
            } catch {
                completionHandler(error)
            }
        }
    }
    
    private func processPacket(_ packet: Data, protocol: NSNumber) -> PacketAction {
        let parsed = core.parsePacket(packet)
        let decision = core.shouldBlock(parsed)
        
        switch decision {
        case .block:
            return .drop
        case .modify(let rules):
            let modified = core.modifyPacket(parsed, rules: rules)
            return .forward(modified)
        case .allow:
            return .forward(packet)
        }
    }
}

enum PacketAction {
    case drop
    case forward(Data)
}
```

### 3.3 Safari Extension API
```swift
// Content Blocker
class ContentBlockerRequestHandler: NSObject, NSExtensionRequestHandling {
    func beginRequest(with context: NSExtensionContext) {
        Task {
            do {
                let rules = try await generateBlockingRules()
                let data = try JSONEncoder().encode(rules)
                let url = FileManager.default.temporaryDirectory
                    .appendingPathComponent("blocklist.json")
                try data.write(to: url)
                
                let attachment = NSItemProvider(contentsOf: url)!
                let item = NSExtensionItem()
                item.attachments = [attachment]
                
                context.completeRequest(returningItems: [item])
            } catch {
                context.cancelRequest(withError: error)
            }
        }
    }
    
    private func generateBlockingRules() async throws -> [ContentBlockerRule] {
        let core = AdBlockCore(config: .default)
        return core.getSafariRules()
    }
}

struct ContentBlockerRule: Codable {
    let trigger: Trigger
    let action: Action
    
    struct Trigger: Codable {
        let urlFilter: String
        let ifDomain: [String]?
        let unlessDomain: [String]?
    }
    
    struct Action: Codable {
        let type: ActionType
        let selector: String?
    }
    
    enum ActionType: String, Codable {
        case block
        case cssDisplayNone = "css-display-none"
        case ignorePreviousRules = "ignore-previous-rules"
    }
}
```

### 3.4 SwiftUI Components
```swift
// View Model
@MainActor
class AdBlockViewModel: ObservableObject {
    @Published var isEnabled = false
    @Published var statistics = Statistics.empty
    @Published var recentEvents: [BlockEvent] = []
    
    private let service: AdBlockServiceProtocol
    private var cancellables = Set<AnyCancellable>()
    
    init(service: AdBlockServiceProtocol) {
        self.service = service
        observeEvents()
    }
    
    func toggleBlocking() {
        Task {
            do {
                if isEnabled {
                    try await service.stop()
                } else {
                    try await service.start()
                }
                isEnabled.toggle()
            } catch {
                // Handle error
            }
        }
    }
    
    private func observeEvents() {
        Task {
            for await event in service.blockEvents {
                await MainActor.run {
                    recentEvents.append(event)
                    if recentEvents.count > 100 {
                        recentEvents.removeFirst()
                    }
                }
            }
        }
    }
}

// SwiftUI View
struct AdBlockView: View {
    @StateObject private var viewModel: AdBlockViewModel
    
    var body: some View {
        NavigationView {
            List {
                // Main toggle
                Section {
                    Toggle("Enable Ad Blocking", isOn: $viewModel.isEnabled)
                        .onChange(of: viewModel.isEnabled) { _ in
                            viewModel.toggleBlocking()
                        }
                }
                
                // Statistics
                Section("Statistics") {
                    StatisticsView(statistics: viewModel.statistics)
                }
                
                // Recent activity
                Section("Recent Activity") {
                    ForEach(viewModel.recentEvents) { event in
                        BlockEventRow(event: event)
                    }
                }
            }
            .navigationTitle("Ad Blocker")
        }
    }
}
```

## 4. 共通データモデル

### 4.1 フィルタールール形式
```json
{
  "version": "1.0.0",
  "rules": [
    {
      "id": "rule_001",
      "type": "url_pattern",
      "pattern": "||doubleclick.net^",
      "action": "block",
      "priority": 1000
    },
    {
      "id": "rule_002", 
      "type": "css_selector",
      "selector": ".ad-container, .sponsored",
      "domains": ["example.com"],
      "action": "hide"
    },
    {
      "id": "rule_003",
      "type": "script_injection",
      "script": "window.adblockEnabled = true;",
      "domains": ["youtube.com"],
      "timing": "document_start"
    }
  ]
}
```

### 4.2 設定スキーマ
```yaml
# config.yaml
version: 1.0
core:
  update_interval: 86400
  cache_size: 100MB
  log_level: info

filtering:
  enabled: true
  youtube_ads: true
  social_media_tracking: true
  analytics: true

network:
  dns_servers:
    - 8.8.8.8
    - 8.8.4.4
  timeout: 5000
  retry_count: 3

privacy:
  telemetry: false
  crash_reports: false
  local_storage_only: true
```

## 5. エラーコード定義

### 5.1 共通エラーコード
```rust
#[repr(C)]
pub enum ErrorCode {
    Success = 0,
    
    // Initialization errors (1000-1999)
    InitializationFailed = 1000,
    InvalidConfiguration = 1001,
    MissingPermissions = 1002,
    
    // Network errors (2000-2999)
    NetworkError = 2000,
    DnsResolutionFailed = 2001,
    ConnectionTimeout = 2002,
    
    // Filter errors (3000-3999)
    InvalidRule = 3000,
    RuleParsingFailed = 3001,
    RuleUpdateFailed = 3002,
    
    // Platform errors (4000-4999)
    VpnEstablishFailed = 4000,
    ExtensionLoadFailed = 4001,
    SystemResourceError = 4002,
}
```

## 6. 非同期通信プロトコル

### 6.1 イベントストリーム
```protobuf
// events.proto
syntax = "proto3";

message BlockEvent {
    int64 timestamp = 1;
    string url = 2;
    string source_app = 3;
    string matched_rule = 4;
    bool blocked = 5;
    int32 size_saved = 6;
}

message StatisticsUpdate {
    int64 total_blocked = 1;
    int64 total_allowed = 2;
    int64 data_saved = 3;
    map<string, int64> domain_counts = 4;
}

message ConfigurationChanged {
    string key = 1;
    string old_value = 2;
    string new_value = 3;
}
```

### 6.2 コマンド・レスポンス
```protobuf
message Command {
    oneof command {
        StartBlocking start = 1;
        StopBlocking stop = 2;
        UpdateRules update_rules = 3;
        GetStatistics get_stats = 4;
    }
}

message Response {
    bool success = 1;
    string error_message = 2;
    oneof data {
        Statistics statistics = 3;
        RuleUpdateResult rule_update = 4;
    }
}
```