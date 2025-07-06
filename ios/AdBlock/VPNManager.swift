import Foundation
import NetworkExtension

class VPNManager: NSObject {
    static let shared = VPNManager()
    
    private var vpnManager: NEVPNManager?
    private var engine: AdBlockEngine?
    
    private override init() {
        super.init()
        loadVPNPreference()
    }
    
    private func loadVPNPreference() {
        NEVPNManager.shared().loadFromPreferences { [weak self] error in
            if let error = error {
                print("Failed to load VPN preferences: \(error)")
            }
            self?.vpnManager = NEVPNManager.shared()
        }
    }
    
    func startVPN(completion: @escaping (Error?) -> Void) {
        guard let vpnManager = vpnManager else {
            completion(NSError(domain: "AdBlock", code: -1, userInfo: [NSLocalizedDescriptionKey: "VPN manager not initialized"]))
            return
        }
        
        // Configure VPN
        let config = NEVPNProtocolIPSec()
        config.serverAddress = "10.0.0.1"
        config.username = "adblock"
        config.authenticationMethod = .none
        config.localIdentifier = "AdBlock"
        config.remoteIdentifier = "AdBlock"
        config.useExtendedAuthentication = false
        config.disconnectOnSleep = false
        
        vpnManager.protocolConfiguration = config
        vpnManager.localizedDescription = "AdBlock VPN"
        vpnManager.isEnabled = true
        vpnManager.isOnDemandEnabled = false
        
        vpnManager.saveToPreferences { [weak self] error in
            if let error = error {
                completion(error)
                return
            }
            
            self?.vpnManager?.loadFromPreferences { error in
                if let error = error {
                    completion(error)
                    return
                }
                
                do {
                    try self?.vpnManager?.connection.startVPNTunnel()
                    completion(nil)
                } catch {
                    completion(error)
                }
            }
        }
    }
    
    func stopVPN() {
        vpnManager?.connection.stopVPNTunnel()
    }
    
    var isConnected: Bool {
        return vpnManager?.connection.status == .connected
    }
    
    func observeVPNStatus(handler: @escaping (NEVPNStatus) -> Void) {
        NotificationCenter.default.addObserver(
            forName: .NEVPNStatusDidChange,
            object: vpnManager?.connection,
            queue: .main
        ) { _ in
            handler(self.vpnManager?.connection.status ?? .disconnected)
        }
    }
}