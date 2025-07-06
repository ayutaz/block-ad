import SwiftUI

struct ContentView: View {
    @State private var isVPNEnabled = false
    @State private var statistics = Statistics(blockedCount: 0, allowedCount: 0, dataSaved: 0)
    @State private var showingSettings = false
    @State private var showingAlert = false
    @State private var alertMessage = ""
    
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
                    .background(Color(.systemBackground))
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
                            Label("フィルター更新", systemImage: "arrow.clockwise")
                                .frame(maxWidth: .infinity)
                        }
                        .buttonStyle(.bordered)
                        
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
        alertMessage = "フィルターリストを更新しました"
        showingAlert = true
    }
    
    private func clearStatistics() {
        statistics = Statistics(blockedCount: 0, allowedCount: 0, dataSaved: 0)
        alertMessage = "統計情報をリセットしました"
        showingAlert = true
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
    @State private var autoUpdate = true
    @State private var blockYouTube = true
    @Environment(\.dismiss) private var dismiss
    
    var body: some View {
        NavigationView {
            Form {
                Section("フィルター設定") {
                    Toggle("フィルターの自動更新", isOn: $autoUpdate)
                    Toggle("YouTube広告ブロック", isOn: $blockYouTube)
                }
                
                Section("情報") {
                    HStack {
                        Text("バージョン")
                        Spacer()
                        Text("1.0.0")
                            .foregroundColor(.secondary)
                    }
                }
            }
            .navigationTitle("設定")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button("閉じる") {
                        dismiss()
                    }
                }
            }
        }
    }
}

struct ContentView_Previews: PreviewProvider {
    static var previews: some View {
        ContentView()
    }
}