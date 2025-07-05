import NetworkExtension

/// Packet tunnel provider for Network Extension
public class AdBlockPacketTunnelProvider: NEPacketTunnelProvider {
    
    public var engine: AdBlockEngine!
    private var pendingPackets: [(Data, Int)] = []
    private let packetQueue = DispatchQueue(label: "com.adblock.packets", attributes: .concurrent)
    
    public override init() {
        super.init()
        self.engine = AdBlockEngine()
    }
    
    /// Load filter rules into the engine
    /// - Parameter rules: Filter rules in EasyList format
    public func loadFilterRules(_ rules: String) {
        _ = engine.loadFilterList(rules)
    }
    
    /// Check if a packet should be blocked based on its destination
    /// - Parameter packet: Mock packet with host and port information
    /// - Returns: true if the packet should be blocked
    public func shouldBlockPacket(_ packet: MockPacket) -> Bool {
        let url = "https://\(packet.host):\(packet.port)"
        return engine.shouldBlock(url)
    }
    
    /// Get current statistics
    /// - Returns: Statistics object with blocking metrics
    public func getStatistics() -> Statistics {
        return engine.getStatistics()
    }
    
    /// Create tunnel configuration
    /// - Returns: NEPacketTunnelNetworkSettings configured for ad blocking
    public func createTunnelConfiguration() -> NEPacketTunnelNetworkSettings {
        let settings = NEPacketTunnelNetworkSettings(tunnelRemoteAddress: "10.0.0.1")
        
        // Configure IPv4 settings
        let ipv4Settings = NEIPv4Settings(addresses: ["10.0.0.2"], subnetMasks: ["255.255.255.0"])
        ipv4Settings.includedRoutes = [NEIPv4Route.default()]
        settings.ipv4Settings = ipv4Settings
        
        // Configure DNS settings
        let dnsSettings = NEDNSSettings(servers: ["8.8.8.8", "8.8.4.4"])
        settings.dnsSettings = dnsSettings
        
        // Set MTU
        settings.mtu = 1500
        
        return settings
    }
    
    override public func startTunnel(options: [String : NSObject]?, completionHandler: @escaping (Error?) -> Void) {
        // Configure tunnel
        let settings = createTunnelConfiguration()
        
        setTunnelNetworkSettings(settings) { error in
            if let error = error {
                completionHandler(error)
                return
            }
            
            // Start reading packets
            self.startPacketHandling()
            completionHandler(nil)
        }
    }
    
    override public func stopTunnel(with reason: NEProviderStopReason, completionHandler: @escaping () -> Void) {
        completionHandler()
    }
    
    override public func handleAppMessage(_ messageData: Data, completionHandler: ((Data?) -> Void)?) {
        // Handle messages from the app
        if let message = String(data: messageData, encoding: .utf8) {
            if message == "stats" {
                let stats = engine.getStatistics()
                let response = """
                {
                    "blocked": \(stats.blockedCount),
                    "allowed": \(stats.allowedCount),
                    "saved": \(stats.dataSaved)
                }
                """
                completionHandler?(response.data(using: .utf8))
            } else if message.hasPrefix("rules:") {
                let rules = String(message.dropFirst(6))
                loadFilterRules(rules)
                completionHandler?("OK".data(using: .utf8))
            } else {
                completionHandler?(nil)
            }
        } else {
            completionHandler?(nil)
        }
    }
    
    private func startPacketHandling() {
        // Read packets from the packet flow
        packetFlow.readPackets { packets, protocols in
            for (index, packet) in packets.enumerated() {
                // Extract destination from packet (simplified for testing)
                if let host = self.extractHost(from: packet) {
                    let mockPacket = MockPacket(host: host, port: 443)
                    
                    if self.shouldBlockPacket(mockPacket) {
                        // Drop the packet
                        continue
                    }
                }
                
                // Forward allowed packets
                self.pendingPackets.append((packet, protocols[index]))
            }
            
            // Write allowed packets
            if !self.pendingPackets.isEmpty {
                let packetsToWrite = self.pendingPackets.map { $0.0 }
                let protocolsToWrite = self.pendingPackets.map { NSNumber(value: $0.1) }
                self.pendingPackets.removeAll()
                
                self.packetFlow.writePackets(packetsToWrite, withProtocols: protocolsToWrite)
            }
            
            // Continue reading
            self.startPacketHandling()
        }
    }
    
    private func extractHost(from packet: Data) -> String? {
        // In a real implementation, parse IP/TCP headers
        // For now, return nil (simplified)
        return nil
    }
}