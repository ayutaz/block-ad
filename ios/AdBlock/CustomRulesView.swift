import SwiftUI

struct CustomRulesView: View {
    @StateObject private var settings = SettingsManager.shared
    @Environment(\.dismiss) private var dismiss
    @State private var rulesText: String = ""
    
    var body: some View {
        NavigationView {
            VStack {
                Text("カスタムフィルタールール")
                    .font(.headline)
                    .padding()
                
                Text("EasyList形式でルールを入力してください")
                    .font(.caption)
                    .foregroundColor(.secondary)
                    .padding(.horizontal)
                
                TextEditor(text: $rulesText)
                    .font(.system(.body, design: .monospaced))
                    .padding(4)
                    .overlay(
                        RoundedRectangle(cornerRadius: 8)
                            .stroke(Color.secondary.opacity(0.3), lineWidth: 1)
                    )
                    .padding()
                
                Text("例: ||example.com/ads^")
                    .font(.caption)
                    .foregroundColor(.secondary)
                    .padding(.horizontal)
                
                Spacer()
            }
            .navigationTitle("カスタムルール")
            #if os(iOS)
            .navigationBarTitleDisplayMode(.inline)
            #endif
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("キャンセル") {
                        dismiss()
                    }
                }
                
                ToolbarItem(placement: .confirmationAction) {
                    Button("保存") {
                        settings.customRules = rulesText
                        dismiss()
                    }
                }
            }
            .onAppear {
                rulesText = settings.customRules
            }
        }
    }
}