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
        guard let manager = vpnManager else {
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
        guard let manager = vpnManager else {
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

// macOS stub implementation
class VPNManager: NSObject {
    static let shared = VPNManager()
    
    var isConnected: Bool {
        return false
    }
    
    private override init() {
        super.init()
    }
    
    func connect() {
        print("VPN connection is not supported on macOS")
    }
    
    func disconnect() {
        print("VPN disconnection is not supported on macOS")
    }
    
    func startVPN(completion: @escaping (Error?) -> Void) {
        print("VPN connection is not supported on macOS")
        completion(nil)
    }
    
    func stopVPN() {
        print("VPN disconnection is not supported on macOS")
    }
}

extension Notification.Name {
    static let vpnStatusDidChange = Notification.Name("VPNStatusDidChange")
}

#endif