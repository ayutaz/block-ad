import Foundation
#if os(iOS)
import NetworkExtension

class VPNManager: NSObject {
    static let shared = VPNManager()
    
    private var vpnManager: NEVPNManager?
    private var vpnStatus: NEVPNStatus = .invalid
    
    var isConnected: Bool {
        return vpnStatus == .connected
    }
    
    private override init() {
        super.init()
        loadVPNConfiguration()
    }
    
    private func loadVPNConfiguration() {
        NEVPNManager.shared().loadFromPreferences { [weak self] error in
            if let error = error {
                print("Failed to load VPN configuration: \(error)")
                return
            }
            
            self?.vpnManager = NEVPNManager.shared()
            self?.vpnStatus = NEVPNManager.shared().connection.status
            
            // Observe VPN status changes
            NotificationCenter.default.addObserver(
                self!,
                selector: #selector(self?.vpnStatusDidChange(_:)),
                name: .NEVPNStatusDidChange,
                object: nil
            )
        }
    }
    
    @objc private func vpnStatusDidChange(_ notification: Notification) {
        guard let connection = notification.object as? NEVPNConnection else { return }
        vpnStatus = connection.status
        
        // Post notification for UI updates
        NotificationCenter.default.post(
            name: .vpnStatusDidChange,
            object: nil,
            userInfo: ["status": vpnStatus.rawValue]
        )
    }
    
    func connect() {
        guard vpnManager != nil else {
            setupVPNConfiguration { [weak self] error in
                if error != nil {
                    print("Failed to setup VPN configuration")
                    return
                }
                self?.startConnection { _ in
                    // Legacy connect method doesn't handle errors
                }
            }
            return
        }
        
        startConnection { _ in
            // Legacy connect method doesn't handle errors
        }
    }
    
    func disconnect() {
        vpnManager?.connection.stopVPNTunnel()
    }
    
    func startVPN(completion: @escaping (Error?) -> Void) {
        guard vpnManager != nil else {
            setupVPNConfiguration { [weak self] error in
                if let error = error {
                    completion(error)
                    return
                }
                self?.startConnection(completion: completion)
            }
            return
        }
        
        startConnection(completion: completion)
    }
    
    func stopVPN() {
        disconnect()
    }
    
    private func setupVPNConfiguration(completion: @escaping (Error?) -> Void) {
        let manager = NEVPNManager.shared()
        
        // Configure VPN protocol for Packet Tunnel Provider
        let protocolConfig = NETunnelProviderProtocol()
        protocolConfig.providerBundleIdentifier = "com.adblock.app.tunnel"
        protocolConfig.serverAddress = "AdBlock Local"
        protocolConfig.disconnectOnSleep = false
        
        manager.protocolConfiguration = protocolConfig
        manager.localizedDescription = "AdBlock VPN"
        manager.isEnabled = true
        
        manager.saveToPreferences { [weak self] error in
            if let error = error {
                print("Failed to save VPN configuration: \(error)")
                completion(error)
                return
            }
            
            self?.vpnManager = manager
            completion(nil)
        }
    }
    
    private func startConnection(completion: @escaping (Error?) -> Void) {
        do {
            try vpnManager?.connection.startVPNTunnel()
            completion(nil)
        } catch {
            print("Failed to start VPN: \(error)")
            completion(error)
        }
    }
}

extension Notification.Name {
    static let vpnStatusDidChange = Notification.Name("VPNStatusDidChange")
}

#else

// macOS implementation
import SystemExtensions
import Network

class VPNManager: NSObject {
    static let shared = VPNManager()
    
    private var providerManager: NETunnelProviderManager?
    private var extensionBundle = "com.adblock.app.networkextension"
    
    var isConnected: Bool {
        return providerManager?.connection.status == .connected
    }
    
    private override init() {
        super.init()
        loadVPNConfiguration()
        
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(vpnStatusDidChange(_:)),
            name: .NEVPNStatusDidChange,
            object: nil
        )
    }
    
    deinit {
        NotificationCenter.default.removeObserver(self)
    }
    
    private func loadVPNConfiguration() {
        NETunnelProviderManager.loadAllFromPreferences { [weak self] managers, error in
            if let error = error {
                print("Failed to load VPN configurations: \(error)")
                return
            }
            
            self?.providerManager = managers?.first ?? NETunnelProviderManager()
            self?.setupVPNConfiguration()
        }
    }
    
    private func setupVPNConfiguration() {
        guard let providerManager = providerManager else { return }
        
        providerManager.localizedDescription = "AdBlock Network Filter"
        
        let providerProtocol = NETunnelProviderProtocol()
        providerProtocol.providerBundleIdentifier = extensionBundle
        providerProtocol.serverAddress = "AdBlock"
        providerProtocol.providerConfiguration = [
            "filterEnabled": true,
            "blockAds": true,
            "blockTrackers": true
        ]
        
        providerManager.protocolConfiguration = providerProtocol
        providerManager.isEnabled = true
        
        // Configure on-demand rules
        let connectRule = NEOnDemandRuleConnect()
        connectRule.interfaceTypeMatch = .any
        providerManager.onDemandRules = [connectRule]
        providerManager.isOnDemandEnabled = false
    }
    
    func connect() {
        // First, ensure System Extension is installed
        SystemExtensionManager.shared.installSystemExtension { [weak self] result in
            switch result {
            case .success:
                self?.connectAfterExtensionInstalled()
            case .failure(let error):
                print("Failed to install System Extension: \(error)")
                // Try to connect anyway in case extension is already installed
                self?.connectAfterExtensionInstalled()
            }
        }
    }
    
    private func connectAfterExtensionInstalled() {
        loadVPNConfiguration()
        
        guard let providerManager = providerManager else {
            print("VPN configuration not loaded")
            return
        }
        
        providerManager.saveToPreferences { [weak self] error in
            if let error = error {
                print("Failed to save VPN configuration: \(error)")
                return
            }
            
            self?.providerManager?.loadFromPreferences { error in
                if let error = error {
                    print("Failed to reload VPN configuration: \(error)")
                    return
                }
                
                do {
                    try self?.providerManager?.connection.startVPNTunnel()
                } catch {
                    print("Failed to start VPN: \(error)")
                }
            }
        }
    }
    
    func disconnect() {
        providerManager?.connection.stopVPNTunnel()
    }
    
    func startVPN(completion: @escaping (Error?) -> Void) {
        connect()
        
        // Monitor connection status
        DispatchQueue.main.asyncAfter(deadline: .now() + 2.0) { [weak self] in
            if self?.isConnected == true {
                completion(nil)
            } else {
                completion(NSError(domain: "VPNManager", code: -1, userInfo: [
                    NSLocalizedDescriptionKey: "Failed to establish VPN connection"
                ]))
            }
        }
    }
    
    func stopVPN() {
        disconnect()
    }
    
    @objc private func vpnStatusDidChange(_ notification: Notification) {
        NotificationCenter.default.post(
            name: .vpnStatusDidChange,
            object: nil,
            userInfo: ["isConnected": isConnected]
        )
    }
}

extension Notification.Name {
    static let vpnStatusDidChange = Notification.Name("VPNStatusDidChange")
}

#endif