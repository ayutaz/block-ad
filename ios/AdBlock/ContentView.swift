import SwiftUI

struct ContentView: View {
    @State private var isVPNEnabled = false
    @State private var statistics = Statistics(blockedCount: 0, allowedCount: 0, dataSaved: 0)
    @State private var showingSettings = false
    @State private var showingAlert = false
    @State private var alertMessage = ""
    @State private var isUpdatingFilters = false
    
    private let engine = try? AdBlockEngine()
    private var filterUpdater: FilterListUpdater? {
        guard let engine = engine else { return nil }
        return FilterListUpdater(engine: engine)
    }
    
    var body: some View {
        NavigationView {
            ScrollView {
                VStack(spacing: 20) {
                    // Main Toggle Card
                    VStack(spacing: 16) {
                        Image(systemName: isVPNEnabled ? "shield.fill" : "shield")
                            .font(.system(size: 64))
                            .foregroundColor(isVPNEnabled ? .blue : .gray)
                        
                        Text(isVPNEnabled ? "保護中" : "保護されていません")
                            .font(.title)
                            .fontWeight(.semibold)
                            .foregroundColor(isVPNEnabled ? .blue : .gray)
                        
                        Toggle(isOn: $isVPNEnabled) {
                            EmptyView()
                        }
                        .toggleStyle(SwitchToggleStyle(tint: .blue))
                        .scaleEffect(1.5)
                        .onChange(of: isVPNEnabled) { newValue in
                            toggleVPN(enabled: newValue)
                        }
                    }
                    .padding(30)
                    .frame(maxWidth: .infinity)
                    #if os(iOS)
                .background(Color(.systemBackground))
                #else
                .background(Color(NSColor.windowBackgroundColor))
                #endif
                    .cornerRadius(16)
                    .shadow(radius: 5)
                    
                    // Statistics Card
                    VStack(alignment: .leading, spacing: 16) {
                        Text("統計情報")
                            .font(.headline)
                            .padding(.bottom, 8)
                        
                        HStack {
                            StatisticItemView(
                                label: "ブロック済み",
                                value: "\(statistics.blockedCount)",
                                color: .red
                            )
                            
                            Spacer()
                            
                            StatisticItemView(
                                label: "許可済み",
                                value: "\(statistics.allowedCount)",
                                color: .blue
                            )
                            
                            Spacer()
                            
                            StatisticItemView(
                                label: "ブロック率",
                                value: "\(Int(statistics.blockRate))%",
                                color: .orange
                            )
                        }
                        
                        ProgressView(value: statistics.blockRate / 100)
                            .progressViewStyle(LinearProgressViewStyle())
                            .tint(.blue)
                        
                        Text("節約データ量: \(formatDataSize(statistics.dataSaved))")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                    .padding()
                    .frame(maxWidth: .infinity)
                    .background(Color(.secondarySystemBackground))
                    .cornerRadius(12)
                    
                    // Quick Actions
                    HStack(spacing: 12) {
                        Button(action: updateFilterLists) {
                            if isUpdatingFilters {
                                ProgressView()
                                    .progressViewStyle(CircularProgressViewStyle())
                                    .scaleEffect(0.8)
                                    .frame(maxWidth: .infinity)
                            } else {
                                Label("フィルター更新", systemImage: "arrow.clockwise")
                                    .frame(maxWidth: .infinity)
                            }
                        }
                        .buttonStyle(.bordered)
                        .disabled(isUpdatingFilters)
                        
                        Button(action: clearStatistics) {
                            Label("統計リセット", systemImage: "trash")
                                .frame(maxWidth: .infinity)
                        }
                        .buttonStyle(.bordered)
                    }
                }
                .padding()
            }
            .navigationTitle("AdBlock")
            .onAppear {
                loadStatistics()
                // Load cached filters on startup
                _ = filterUpdater?.loadCachedFilters()
            }
            .onReceive(NotificationCenter.default.publisher(for: .vpnStatusDidChange)) { _ in
                // Update statistics when VPN status changes
                updateStatistics()
            }
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button(action: { showingSettings = true }) {
                        Image(systemName: "gear")
                    }
                }
            }
            .sheet(isPresented: $showingSettings) {
                SettingsView()
            }
            .alert("通知", isPresented: $showingAlert) {
                Button("OK") { }
            } message: {
                Text(alertMessage)
            }
        }
    }
    
    private func toggleVPN(enabled: Bool) {
        if enabled {
            // Start VPN
            VPNManager.shared.startVPN { error in
                if let error = error {
                    alertMessage = "VPNの開始に失敗しました: \(error.localizedDescription)"
                    showingAlert = true
                    isVPNEnabled = false
                }
            }
        } else {
            // Stop VPN
            VPNManager.shared.stopVPN()
        }
    }
    
    private func updateFilterLists() {
        guard let updater = filterUpdater else {
            alertMessage = "エンジンの初期化に失敗しました"
            showingAlert = true
            return
        }
        
        isUpdatingFilters = true
        updater.updateFilterLists { result in
            DispatchQueue.main.async {
                isUpdatingFilters = false
                switch result {
                case .success(let message):
                    alertMessage = message
                case .failure(let error):
                    alertMessage = "更新エラー: \(error.localizedDescription)"
                }
                showingAlert = true
            }
        }
    }
    
    private func clearStatistics() {
        statistics = Statistics(blockedCount: 0, allowedCount: 0, dataSaved: 0)
        saveStatistics()
        
        // Reset engine statistics if available
        if let engine = engine {
            engine.resetStatistics()
        }
        
        alertMessage = "統計情報をリセットしました"
        showingAlert = true
    }
    
    private func loadStatistics() {
        if let data = UserDefaults.standard.data(forKey: "adblock_statistics"),
           let decoded = try? JSONDecoder().decode(Statistics.self, from: data) {
            statistics = decoded
        }
    }
    
    private func saveStatistics() {
        if let encoded = try? JSONEncoder().encode(statistics) {
            UserDefaults.standard.set(encoded, forKey: "adblock_statistics")
        }
    }
    
    private func updateStatistics() {
        guard let engine = engine else { return }
        statistics = engine.getStatistics()
        saveStatistics()
    }
    
    private func formatDataSize(_ bytes: Int) -> String {
        let formatter = ByteCountFormatter()
        formatter.countStyle = .binary
        return formatter.string(fromByteCount: Int64(bytes))
    }
}

struct StatisticItemView: View {
    let label: String
    let value: String
    let color: Color
    
    var body: some View {
        VStack {
            Text(value)
                .font(.title2)
                .fontWeight(.bold)
                .foregroundColor(color)
            
            Text(label)
                .font(.caption)
                .foregroundColor(.secondary)
        }
    }
}

struct SettingsView: View {
    @StateObject private var settings = SettingsManager.shared
    @State private var showingCustomRules = false
    @Environment(\.dismiss) private var dismiss
    
    var body: some View {
        NavigationView {
            Form {
                Section("フィルター設定") {
                    Toggle("フィルターの自動更新", isOn: $settings.autoUpdateFilters)
                    Toggle("YouTube広告ブロック", isOn: $settings.blockYouTubeAds)
                    Toggle("DNSブロッキング", isOn: $settings.enableDNSBlocking)
                    
                    Button("カスタムルール") {
                        showingCustomRules = true
                    }
                }
                
                Section("情報") {
                    HStack {
                        Text("バージョン")
                        Spacer()
                        Text("1.0.0")
                            .foregroundColor(.secondary)
                    }
                }
                
                Section {
                    Button("設定をリセット", role: .destructive) {
                        settings.resetToDefaults()
                    }
                }
            }
            .navigationTitle("設定")
            .sheet(isPresented: $showingCustomRules) {
                CustomRulesView()
            }
            #if os(iOS)
            .navigationBarTitleDisplayMode(.inline)
            #endif
            .toolbar {
                #if os(iOS)
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button("閉じる") {
                        dismiss()
                    }
                }
                #else
                ToolbarItem(placement: .cancellationAction) {
                    Button("閉じる") {
                        dismiss()
                    }
                }
                #endif
            }
        }
    }
}

struct ContentView_Previews: PreviewProvider {
    static var previews: some View {
        ContentView()
    }
}