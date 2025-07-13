import NetworkExtension
import Network
import os.log

class PacketTunnelProvider: NEPacketTunnelProvider {
    
    private let logger = Logger(subsystem: "com.adblock.networkextension", category: "PacketTunnel")
    private var packetFlow: NEPacketTunnelFlow?
    private var engine: AdBlockEngine?
    private let dnsPort: UInt16 = 53
    
    override func startTunnel(options: [String : NSObject]?, completionHandler: @escaping (Error?) -> Void) {
        logger.info("Starting AdBlock tunnel for macOS")
        
        // Initialize the ad blocking engine
        engine = AdBlockEngine()
        
        // Load filter rules
        if let filterListUpdater = FilterListUpdater(engine: engine!) {
            _ = filterListUpdater.loadCachedFilters()
        }
        
        // Configure tunnel settings
        let settings = NEPacketTunnelNetworkSettings(tunnelRemoteAddress: "127.0.0.1")
        
        // Configure IPv4 settings
        let ipv4Settings = NEIPv4Settings(addresses: ["10.0.0.1"], subnetMasks: ["255.255.255.0"])
        ipv4Settings.includedRoutes = [NEIPv4Route.default()]
        settings.ipv4Settings = ipv4Settings
        
        // Configure DNS settings to intercept DNS queries
        let dnsSettings = NEDNSSettings(servers: ["10.0.0.1"])
        dnsSettings.matchDomains = [""]
        settings.dnsSettings = dnsSettings
        
        // Apply tunnel settings
        setTunnelNetworkSettings(settings) { [weak self] error in
            if let error = error {
                self?.logger.error("Failed to set tunnel settings: \(error.localizedDescription)")
                completionHandler(error)
                return
            }
            
            self?.logger.info("Tunnel settings applied successfully")
            self?.packetFlow = self?.packetFlow
            
            // Start reading packets
            self?.startReadingPackets()
            
            completionHandler(nil)
        }
    }
    
    override func stopTunnel(with reason: NEProviderStopReason, completionHandler: @escaping () -> Void) {
        logger.info("Stopping AdBlock tunnel: \(String(describing: reason))")
        completionHandler()
    }
    
    override func handleAppMessage(_ messageData: Data, completionHandler: ((Data?) -> Void)?) {
        // Handle messages from the app
        logger.info("Received app message")
        
        if let message = String(data: messageData, encoding: .utf8) {
            switch message {
            case "getStats":
                if let stats = engine?.getStatistics() {
                    let response = """
                    {
                        "blocked": \(stats.adsBlocked),
                        "allowed": \(stats.requestsAllowed),
                        "saved": \(stats.dataSaved)
                    }
                    """.data(using: .utf8)
                    completionHandler?(response)
                } else {
                    completionHandler?(nil)
                }
            case "reloadFilters":
                if let filterListUpdater = FilterListUpdater(engine: engine!) {
                    _ = filterListUpdater.loadCachedFilters()
                    completionHandler?("OK".data(using: .utf8))
                } else {
                    completionHandler?(nil)
                }
            default:
                completionHandler?(nil)
            }
        }
    }
    
    private func startReadingPackets() {
        packetFlow?.readPackets { [weak self] packets, protocols in
            self?.handlePackets(packets, protocols: protocols)
            self?.startReadingPackets() // Continue reading
        }
    }
    
    private func handlePackets(_ packets: [Data], protocols: [NSNumber]) {
        var packetsToForward: [Data] = []
        var protocolsToForward: [NSNumber] = []
        
        for (index, packet) in packets.enumerated() {
            let protocolFamily = protocols[index].uint32Value
            
            if protocolFamily == AF_INET {
                // IPv4 packet
                if let modifiedPacket = processIPv4Packet(packet) {
                    packetsToForward.append(modifiedPacket)
                    protocolsToForward.append(protocols[index])
                }
            } else if protocolFamily == AF_INET6 {
                // IPv6 packet
                if let modifiedPacket = processIPv6Packet(packet) {
                    packetsToForward.append(modifiedPacket)
                    protocolsToForward.append(protocols[index])
                }
            } else {
                // Unknown protocol, forward as-is
                packetsToForward.append(packet)
                protocolsToForward.append(protocols[index])
            }
        }
        
        // Write packets back
        if !packetsToForward.isEmpty {
            packetFlow?.writePackets(packetsToForward, withProtocols: protocolsToForward)
        }
    }
    
    private func processIPv4Packet(_ packet: Data) -> Data? {
        guard packet.count >= 20 else { return packet }
        
        // Extract IP header info
        let ipVersion = (packet[0] >> 4) & 0x0F
        guard ipVersion == 4 else { return packet }
        
        let ipHeaderLength = Int(packet[0] & 0x0F) * 4
        guard packet.count >= ipHeaderLength else { return packet }
        
        let protocol = packet[9]
        
        // Check if it's UDP (17) or TCP (6)
        if protocol == 17 { // UDP
            guard packet.count >= ipHeaderLength + 8 else { return packet }
            
            let destPort = UInt16(packet[ipHeaderLength + 2]) << 8 | UInt16(packet[ipHeaderLength + 3])
            
            if destPort == dnsPort {
                // This is a DNS query, process it
                return processDNSPacket(packet, ipHeaderLength: ipHeaderLength)
            }
        } else if protocol == 6 { // TCP
            // Extract destination IP for HTTP/HTTPS filtering
            let destIP = extractDestinationIP(from: packet, headerLength: ipHeaderLength)
            
            if shouldBlockIP(destIP) {
                logger.debug("Blocking TCP connection to: \(destIP)")
                return nil // Drop the packet
            }
        }
        
        return packet
    }
    
    private func processIPv6Packet(_ packet: Data) -> Data? {
        guard packet.count >= 40 else { return packet }
        
        let nextHeader = packet[6]
        
        // Similar processing for IPv6
        if nextHeader == 17 { // UDP
            let destPort = UInt16(packet[40 + 2]) << 8 | UInt16(packet[40 + 3])
            
            if destPort == dnsPort {
                return processDNSPacket(packet, ipHeaderLength: 40)
            }
        }
        
        return packet
    }
    
    private func processDNSPacket(_ packet: Data, ipHeaderLength: Int) -> Data? {
        guard packet.count > ipHeaderLength + 8 + 12 else { return packet }
        
        let udpHeaderLength = 8
        let dnsDataStart = ipHeaderLength + udpHeaderLength
        
        // Extract domain name from DNS query
        if let domain = extractDomainFromDNS(packet, offset: dnsDataStart + 12) {
            logger.debug("DNS query for: \(domain)")
            
            if engine?.shouldBlock(domain) == true {
                logger.info("Blocking DNS query for: \(domain)")
                // Return a DNS response with NXDOMAIN
                return createBlockedDNSResponse(for: packet, ipHeaderLength: ipHeaderLength)
            }
        }
        
        return packet
    }
    
    private func extractDomainFromDNS(_ packet: Data, offset: Int) -> String? {
        var domain = ""
        var pos = offset
        
        while pos < packet.count {
            let length = Int(packet[pos])
            if length == 0 {
                break
            }
            
            if length > 63 || pos + length >= packet.count {
                return nil
            }
            
            if !domain.isEmpty {
                domain += "."
            }
            
            for i in 1...length {
                domain += String(Character(UnicodeScalar(packet[pos + i])))
            }
            
            pos += length + 1
        }
        
        return domain.isEmpty ? nil : domain
    }
    
    private func createBlockedDNSResponse(for packet: Data, ipHeaderLength: Int) -> Data {
        var response = packet
        
        // Set DNS response flags (QR=1, RCODE=3 for NXDOMAIN)
        let flagsOffset = ipHeaderLength + 8 + 2
        if flagsOffset + 1 < response.count {
            response[flagsOffset] = 0x81     // QR=1, Opcode=0, AA=0, TC=0, RD=1
            response[flagsOffset + 1] = 0x83 // RA=1, Z=0, RCODE=3 (NXDOMAIN)
        }
        
        return response
    }
    
    private func extractDestinationIP(from packet: Data, headerLength: Int) -> String {
        guard packet.count >= headerLength else { return "" }
        
        let destIPStart = 16
        let destIP = "\(packet[destIPStart]).\(packet[destIPStart + 1]).\(packet[destIPStart + 2]).\(packet[destIPStart + 3])"
        
        return destIP
    }
    
    private func shouldBlockIP(_ ip: String) -> Bool {
        // Check against known ad server IPs
        let blockedIPs = [
            "0.0.0.0",
            "127.0.0.1"
            // Add more blocked IPs as needed
        ]
        
        return blockedIPs.contains(ip)
    }
}