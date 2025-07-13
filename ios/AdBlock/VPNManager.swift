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
            setupVPNConfiguration { [weak self] in
                self?.startConnection()
            }
            return
        }
        
        startConnection()
    }
    
    func disconnect() {
        vpnManager?.connection.stopVPNTunnel()
    }
    
    private func setupVPNConfiguration(completion: @escaping () -> Void) {
        let manager = NEVPNManager.shared()
        
        // Configure VPN protocol
        let protocolConfig = NEVPNProtocolIPSec()
        protocolConfig.serverAddress = "AdBlock Local"
        protocolConfig.username = "adblock"
        protocolConfig.passwordReference = nil
        protocolConfig.authenticationMethod = .none
        protocolConfig.useExtendedAuthentication = false
        protocolConfig.disconnectOnSleep = false
        
        manager.protocolConfiguration = protocolConfig
        manager.localizedDescription = "AdBlock VPN"
        manager.isEnabled = true
        
        manager.saveToPreferences { [weak self] error in
            if let error = error {
                print("Failed to save VPN configuration: \(error)")
                return
            }
            
            self?.vpnManager = manager
            completion()
        }
    }
    
    private func startConnection() {
        do {
            try vpnManager?.connection.startVPNTunnel()
        } catch {
            print("Failed to start VPN: \(error)")
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
}

extension Notification.Name {
    static let vpnStatusDidChange = Notification.Name("VPNStatusDidChange")
}

#endif