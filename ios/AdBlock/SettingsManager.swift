import Foundation

/// Manages app settings and preferences
public class SettingsManager: ObservableObject {
    static let shared = SettingsManager()
    
    @Published var autoUpdateFilters: Bool {
        didSet {
            UserDefaults.standard.set(autoUpdateFilters, forKey: "auto_update_filters")
            BackgroundTaskManager.shared.setAutoUpdateEnabled(autoUpdateFilters)
        }
    }
    
    @Published var blockYouTubeAds: Bool {
        didSet {
            UserDefaults.standard.set(blockYouTubeAds, forKey: "block_youtube_ads")
        }
    }
    
    @Published var enableDNSBlocking: Bool {
        didSet {
            UserDefaults.standard.set(enableDNSBlocking, forKey: "enable_dns_blocking")
        }
    }
    
    @Published var customRules: String {
        didSet {
            UserDefaults.standard.set(customRules, forKey: "custom_rules")
        }
    }
    
    private init() {
        // Load saved settings
        self.autoUpdateFilters = UserDefaults.standard.bool(forKey: "auto_update_filters")
        self.blockYouTubeAds = UserDefaults.standard.bool(forKey: "block_youtube_ads")
        self.enableDNSBlocking = UserDefaults.standard.bool(forKey: "enable_dns_blocking")
        self.customRules = UserDefaults.standard.string(forKey: "custom_rules") ?? ""
        
        // Set defaults if first launch
        if !UserDefaults.standard.bool(forKey: "has_launched_before") {
            self.autoUpdateFilters = true
            self.blockYouTubeAds = true
            self.enableDNSBlocking = true
            UserDefaults.standard.set(true, forKey: "has_launched_before")
        }
    }
    
    /// Get combined filter rules including custom rules
    public func getCombinedRules() -> String {
        var rules = ""
        
        // Add YouTube-specific rules if enabled
        if blockYouTubeAds {
            rules += """
            ||youtube.com/api/stats/ads^
            ||youtube.com/pagead^
            ||youtube.com/ptracking^
            ||googlevideo.com/videoplayback*ctier^
            ||googlevideo.com/initplayback^
            ||googlevideo.com/ptracking^
            
            """
        }
        
        // Add custom rules
        if !customRules.isEmpty {
            rules += customRules
        }
        
        return rules
    }
    
    /// Reset all settings to defaults
    public func resetToDefaults() {
        autoUpdateFilters = true
        blockYouTubeAds = true
        enableDNSBlocking = true
        customRules = ""
    }
}