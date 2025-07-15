import SwiftUI

struct EnhancedCustomRulesView: View {
    @Environment(\.dismiss) private var dismiss
    @StateObject private var settings = SettingsManager.shared
    @State private var rules: [CustomRule] = []
    @State private var showAddRule = false
    @State private var showImportRules = false
    @State private var selectedRule: CustomRule?
    @State private var searchText = ""
    
    var filteredRules: [CustomRule] {
        if searchText.isEmpty {
            return rules
        }
        return rules.filter { $0.rule.localizedCaseInsensitiveContains(searchText) }
    }
    
    var body: some View {
        NavigationView {
            List {
                Section {
                    HStack {
                        Image(systemName: "info.circle.fill")
                            .foregroundColor(.blue)
                        Text("EasyList形式のフィルタールールを追加できます")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                    .padding(.vertical, 4)
                }
                
                if rules.isEmpty {
                    Section {
                        VStack(spacing: 12) {
                            Image(systemName: "doc.text.magnifyingglass")
                                .font(.largeTitle)
                                .foregroundColor(.secondary)
                            Text("カスタムルールがありません")
                                .font(.headline)
                            Text("＋ボタンから追加してください")
                                .font(.caption)
                                .foregroundColor(.secondary)
                        }
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, 20)
                    }
                } else {
                    Section(header: Text("ルール (\(filteredRules.count))")) {
                        ForEach(filteredRules) { rule in
                            RuleRow(rule: rule) {
                                selectedRule = rule
                            }
                        }
                        .onDelete(perform: deleteRules)
                    }
                }
            }
            .searchable(text: $searchText, prompt: "ルールを検索")
            .navigationTitle("カスタムフィルタールール")
            .navigationBarTitleDisplayMode(.large)
            .toolbar {
                ToolbarItem(placement: .navigationBarLeading) {
                    Button("キャンセル") {
                        dismiss()
                    }
                }
                
                ToolbarItem(placement: .navigationBarTrailing) {
                    Menu {
                        Button(action: { showAddRule = true }) {
                            Label("新規ルール", systemImage: "plus")
                        }
                        Button(action: { showImportRules = true }) {
                            Label("インポート", systemImage: "square.and.arrow.down")
                        }
                        Button(action: exportRules) {
                            Label("エクスポート", systemImage: "square.and.arrow.up")
                        }
                    } label: {
                        Image(systemName: "ellipsis.circle")
                    }
                }
            }
            .sheet(isPresented: $showAddRule) {
                AddRuleView { newRule in
                    addRule(newRule)
                }
            }
            .sheet(isPresented: $showImportRules) {
                ImportRulesView { importedRules in
                    importRules(importedRules)
                }
            }
            .sheet(item: $selectedRule) { rule in
                RuleDetailView(rule: rule) {
                    deleteRule(rule)
                }
            }
            .onAppear {
                loadRules()
            }
            .onDisappear {
                saveRules()
            }
        }
    }
    
    private func loadRules() {
        let rulesText = settings.customRules
        rules = rulesText.split(separator: "\n")
            .map { String($0).trimmingCharacters(in: .whitespaces) }
            .filter { !$0.isEmpty }
            .map { CustomRule(rule: $0) }
    }
    
    private func saveRules() {
        let rulesText = rules.map { $0.rule }.joined(separator: "\n")
        settings.customRules = rulesText
        
        // Apply to engine
        if let engine = AdBlockEngine.shared {
            _ = engine.loadFilterList(rulesText)
        }
    }
    
    private func addRule(_ rule: String) {
        let trimmedRule = rule.trimmingCharacters(in: .whitespaces)
        guard !trimmedRule.isEmpty else { return }
        
        // Check for duplicates
        if !rules.contains(where: { $0.rule == trimmedRule }) {
            rules.append(CustomRule(rule: trimmedRule))
            saveRules()
        }
    }
    
    private func deleteRules(at offsets: IndexSet) {
        rules.remove(atOffsets: offsets)
        saveRules()
    }
    
    private func deleteRule(_ rule: CustomRule) {
        rules.removeAll { $0.id == rule.id }
        saveRules()
    }
    
    private func importRules(_ importedRules: [String]) {
        for rule in importedRules {
            addRule(rule)
        }
    }
    
    private func exportRules() {
        let rulesText = rules.map { $0.rule }.joined(separator: "\n")
        let activityVC = UIActivityViewController(
            activityItems: [rulesText],
            applicationActivities: nil
        )
        
        if let windowScene = UIApplication.shared.connectedScenes.first as? UIWindowScene,
           let window = windowScene.windows.first {
            window.rootViewController?.present(activityVC, animated: true)
        }
    }
}

struct CustomRule: Identifiable {
    let id = UUID()
    let rule: String
    
    var type: RuleType {
        if rule.hasPrefix("||") && rule.hasSuffix("^") {
            return .domain
        } else if rule.hasPrefix("@@") {
            return .exception
        } else if rule.hasPrefix("##") {
            return .element
        } else if rule.contains("*") {
            return .wildcard
        } else {
            return .custom
        }
    }
    
    var description: String {
        switch type {
        case .domain:
            return "ドメインブロック"
        case .exception:
            return "例外ルール"
        case .element:
            return "要素非表示"
        case .wildcard:
            return "ワイルドカード"
        case .custom:
            return "カスタムルール"
        }
    }
    
    var icon: String {
        switch type {
        case .domain:
            return "network.slash"
        case .exception:
            return "checkmark.shield"
        case .element:
            return "eye.slash"
        case .wildcard:
            return "asterisk"
        case .custom:
            return "doc.text"
        }
    }
    
    var color: Color {
        switch type {
        case .domain:
            return .red
        case .exception:
            return .green
        case .element:
            return .orange
        case .wildcard:
            return .purple
        case .custom:
            return .blue
        }
    }
    
    enum RuleType {
        case domain
        case exception
        case element
        case wildcard
        case custom
    }
}

struct RuleRow: View {
    let rule: CustomRule
    let onTap: () -> Void
    
    var body: some View {
        Button(action: onTap) {
            HStack {
                Image(systemName: rule.icon)
                    .foregroundColor(rule.color)
                    .frame(width: 24)
                
                VStack(alignment: .leading, spacing: 2) {
                    Text(rule.rule)
                        .font(.system(.body, design: .monospaced))
                        .lineLimit(1)
                        .truncationMode(.middle)
                        .foregroundColor(.primary)
                    
                    Text(rule.description)
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                
                Spacer()
                
                Image(systemName: "chevron.right")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
            .padding(.vertical, 4)
        }
        .buttonStyle(PlainButtonStyle())
    }
}

struct AddRuleView: View {
    @Environment(\.dismiss) private var dismiss
    @State private var ruleText = ""
    @State private var ruleType = RuleType.domain
    let onAdd: (String) -> Void
    
    enum RuleType: String, CaseIterable {
        case domain = "ドメイン"
        case exception = "例外"
        case custom = "カスタム"
        
        var placeholder: String {
            switch self {
            case .domain:
                return "example.com"
            case .exception:
                return "example.com/allowed"
            case .custom:
                return "||example.com^"
            }
        }
        
        var hint: String {
            switch self {
            case .domain:
                return "入力されたドメインは ||domain^ 形式に変換されます"
            case .exception:
                return "入力されたURLは @@||url 形式に変換されます"
            case .custom:
                return "EasyList形式のルールを直接入力してください"
            }
        }
    }
    
    var body: some View {
        NavigationView {
            Form {
                Section(header: Text("ルールタイプ")) {
                    Picker("タイプ", selection: $ruleType) {
                        ForEach(RuleType.allCases, id: \.self) { type in
                            Text(type.rawValue).tag(type)
                        }
                    }
                    .pickerStyle(SegmentedPickerStyle())
                }
                
                Section(header: Text("ルール"), footer: Text(ruleType.hint)) {
                    TextField(ruleType.placeholder, text: $ruleText)
                        .textFieldStyle(RoundedBorderTextFieldStyle())
                        .autocapitalization(.none)
                        .disableAutocorrection(true)
                }
                
                Section {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("ルール例:")
                            .font(.caption)
                            .foregroundColor(.secondary)
                        
                        VStack(alignment: .leading, spacing: 4) {
                            Text("||example.com^ - ドメインをブロック")
                            Text("@@||example.com - 例外として許可")
                            Text("##.ad-banner - 要素を非表示")
                            Text("*/ads/* - パターンマッチング")
                        }
                        .font(.caption2)
                        .foregroundColor(.secondary)
                    }
                }
            }
            .navigationTitle("新規ルール")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .navigationBarLeading) {
                    Button("キャンセル") {
                        dismiss()
                    }
                }
                
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button("追加") {
                        let finalRule: String
                        switch ruleType {
                        case .domain:
                            finalRule = "||\\(ruleText.trimmingCharacters(in: .whitespaces))^"
                        case .exception:
                            finalRule = "@@||\\(ruleText.trimmingCharacters(in: .whitespaces))"
                        case .custom:
                            finalRule = ruleText.trimmingCharacters(in: .whitespaces)
                        }
                        
                        if !finalRule.isEmpty {
                            onAdd(finalRule)
                            dismiss()
                        }
                    }
                    .disabled(ruleText.trimmingCharacters(in: .whitespaces).isEmpty)
                }
            }
        }
    }
}

struct ImportRulesView: View {
    @Environment(\.dismiss) private var dismiss
    @State private var rulesText = ""
    let onImport: ([String]) -> Void
    
    var body: some View {
        NavigationView {
            Form {
                Section(header: Text("フィルタールールをインポート"),
                       footer: Text("複数のルールを改行で区切って入力してください")) {
                    TextEditor(text: $rulesText)
                        .font(.system(.body, design: .monospaced))
                        .frame(minHeight: 200)
                }
                
                Section {
                    Button(action: pasteFromClipboard) {
                        Label("クリップボードから貼り付け", systemImage: "doc.on.clipboard")
                    }
                }
            }
            .navigationTitle("ルールをインポート")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .navigationBarLeading) {
                    Button("キャンセル") {
                        dismiss()
                    }
                }
                
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button("インポート") {
                        let rules = rulesText
                            .split(separator: "\n")
                            .map { $0.trimmingCharacters(in: .whitespaces) }
                            .filter { !$0.isEmpty }
                        
                        if !rules.isEmpty {
                            onImport(rules)
                            dismiss()
                        }
                    }
                    .disabled(rulesText.trimmingCharacters(in: .whitespaces).isEmpty)
                }
            }
        }
    }
    
    private func pasteFromClipboard() {
        if let clipboard = UIPasteboard.general.string {
            rulesText = clipboard
        }
    }
}

struct RuleDetailView: View {
    let rule: CustomRule
    let onDelete: () -> Void
    @Environment(\.dismiss) private var dismiss
    
    var body: some View {
        NavigationView {
            List {
                Section(header: Text("ルール")) {
                    HStack {
                        Image(systemName: rule.icon)
                            .foregroundColor(rule.color)
                        Text(rule.rule)
                            .font(.system(.body, design: .monospaced))
                    }
                }
                
                Section(header: Text("タイプ")) {
                    HStack {
                        Text(rule.description)
                        Spacer()
                        Image(systemName: rule.icon)
                            .foregroundColor(rule.color)
                    }
                }
                
                Section(header: Text("説明")) {
                    Text(ruleExplanation)
                        .font(.body)
                        .foregroundColor(.secondary)
                }
                
                Section {
                    Button(action: {
                        UIPasteboard.general.string = rule.rule
                        dismiss()
                    }) {
                        Label("ルールをコピー", systemImage: "doc.on.doc")
                    }
                    
                    Button(role: .destructive, action: {
                        onDelete()
                        dismiss()
                    }) {
                        Label("削除", systemImage: "trash")
                            .foregroundColor(.red)
                    }
                }
            }
            .navigationTitle("ルールの詳細")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button("完了") {
                        dismiss()
                    }
                }
            }
        }
    }
    
    private var ruleExplanation: String {
        switch rule.type {
        case .domain:
            let domain = rule.rule
                .dropFirst(2)
                .dropLast(1)
            return "「\(domain)」ドメインからのすべてのリクエストをブロックします。"
        case .exception:
            let pattern = rule.rule.dropFirst(2)
            return "「\(pattern)」にマッチするリクエストを例外として許可します。"
        case .element:
            return "ページ内の特定の要素を非表示にします。"
        case .wildcard:
            return "ワイルドカード（*）を使用してパターンマッチングを行います。"
        case .custom:
            return "カスタムフィルタールールです。"
        }
    }
}

// Extension for AdBlockEngine
extension AdBlockEngine {
    static var shared: AdBlockEngine? {
        // This should be managed by the app
        return nil
    }
}