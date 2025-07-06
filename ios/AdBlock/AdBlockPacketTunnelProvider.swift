import NetworkExtension

/// Packet tunnel provider for Network Extension
public class AdBlockPacketTunnelProvider: NEPacketTunnelProvider {
    
    public private(set) var engine: AdBlockEngine?
    private let packetQueue = DispatchQueue(label: "com.adblock.packets", attributes: .concurrent)
    
    public override init() {
        super.init()
        
        do {
            self.engine = try AdBlockEngine()
            loadDefaultFilterLists()
        } catch {
            // Log the error - in production, you'd want proper error handling here
            NSLog("Failed to initialize AdBlockEngine: \(error)")
        }
    }
    
    /// Load filter rules into the engine
    /// - Parameter rules: Filter rules in EasyList format
    public func loadFilterRules(_ rules: String) {
        _ = engine?.loadFilterList(rules)
    }
    
    /// Check if a packet should be blocked based on its destination
    /// - Parameter packet: Mock packet with host and port information
    /// - Returns: true if the packet should be blocked
    public func shouldBlockPacket(_ packet: MockPacket) -> Bool {
        guard let engine = engine else { return false }
        let url = "https://\(packet.host):\(packet.port)"
        return engine.shouldBlock(url)
    }
    
    /// Get current statistics
    /// - Returns: Statistics object with blocking metrics
    public func getStatistics() -> Statistics {
        return engine?.getStatistics() ?? Statistics(blockedCount: 0, allowedCount: 0, dataSaved: 0)
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
                let stats = getStatistics()
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
        packetFlow.readPackets { [weak self] packets, protocols in
            guard let self = self else { return }
            
            var allowedPackets: [(Data, NSNumber)] = []
            
            for (index, packet) in packets.enumerated() {
                // Extract destination from packet (simplified for testing)
                if let host = self.extractHost(from: packet) {
                    let mockPacket = MockPacket(host: host, port: 443)
                    
                    if self.shouldBlockPacket(mockPacket) {
                        // Drop the packet
                        continue
                    }
                }
                
                // Add to allowed packets
                allowedPackets.append((packet, NSNumber(value: protocols[index])))
            }
            
            // Write allowed packets
            if !allowedPackets.isEmpty {
                let packetsToWrite = allowedPackets.map { $0.0 }
                let protocolsToWrite = allowedPackets.map { $0.1 }
                
                self.packetFlow.writePackets(packetsToWrite, withProtocols: protocolsToWrite)
            }
            
            // Continue reading
            self.startPacketHandling()
        }
    }
    
    private func extractHost(from packet: Data) -> String? {
        guard packet.count >= 20 else { return nil }
        
        return packet.withUnsafeBytes { buffer in
            // Check IP version (IPv4 only for now)
            let ipVersion = (buffer[0] >> 4) & 0xF
            guard ipVersion == 4 else { return nil }
            
            // Get protocol (6=TCP, 17=UDP)
            let proto = buffer[9]
            guard proto == 6 || proto == 17 else { return nil }
            
            // Get destination IP
            let destIP = Data(bytes: buffer.baseAddress!.advanced(by: 16), count: 4)
            let ipString = destIP.map { String($0) }.joined(separator: ".")
            
            // Get IP header length
            let ipHeaderLength = Int(buffer[0] & 0xF) * 4
            
            // Get destination port
            guard packet.count >= ipHeaderLength + 4 else { return nil }
            let portBytes = packet.subdata(in: (ipHeaderLength + 2)..<(ipHeaderLength + 4))
            let port = UInt16(portBytes[0]) << 8 | UInt16(portBytes[1])
            
            return ipString
        }
    }
    
    private func loadDefaultFilterLists() {
        let defaultRules = """
            ||doubleclick.net^
            ||googleadservices.com^
            ||googlesyndication.com^
            ||google-analytics.com^
            ||googletagmanager.com^
            ||facebook.com/tr^
            ||amazon-adsystem.com^
            ||adsrvr.org^
            ||adsymptotic.com^
            ||adnxs.com^
            ||adsafeprotected.com^
            ||smaato.net^
            ||smartadserver.com^
            ||scorecardresearch.com^
            ||outbrain.com^
            ||taboola.com^
            ||criteo.com^
            ||criteo.net^
            ||casalemedia.com^
            ||appnexus.com^
            ||rubiconproject.com^
            ||pubmatic.com^
            ||openx.net^
            ||chartboost.com^
            ||unity3d.com/services/ads^
            ||mopub.com^
            ||inmobi.com^
            ||flurry.com^
            ||applovin.com^
            ||startapp.com^
            ||supersonicads.com^
            ||ironsrc.com^
            ||adcolony.com^
            ||vungle.com^
            ||tapjoy.com^
            ||moatads.com^
            ||doubleverify.com^
            ||branch.io^
            ||adjust.com^
            ||kochava.com^
            ||tenjin.io^
            ||singular.net^
            ||appsflyer.com^
            ||crashlytics.com^
            ||fabric.io^
            ||firebase.com/analytics^
            ||mixpanel.com^
            ||segment.com^
            ||amplitude.com^
            ||urbanairship.com^
            ||braze.com^
            ||onesignal.com^
            ||batch.com^
            ||swrve.com^
            ||leanplum.com^
            ||clevertap.com^
            ||airship.com^
            ||mparticle.com^
            ||tune.com^
            ||youappi.com^
            ||bidmachine.io^
            ||admost.com^
            ||bytedance.com/ad^
            ||tiktok.com/ads^
            
            # YouTube specific rules
            ||youtube.com/api/stats/ads^
            ||youtube.com/pagead^
            ||youtube.com/ptracking^
            ||youtube.com/get_video_info*ad^
            ||youtube.com/api/stats/qoe^
            ||googlevideo.com/videoplayback*ctier^
            ||googlevideo.com/initplayback^
            ||googlevideo.com/ptracking^
            ||googlevideo.com/videogoodput^
            ||youtube.com/youtubei/v1/log_event^
            ||youtube.com/youtubei/v1/player/ad_break^
            ||youtube.com/youtubei/v1/next*adplacements^
            ||youtube.com/youtubei/v1/player*adplacements^
            ||googleads.g.doubleclick.net/pagead/id^
            ||googleads.g.doubleclick.net/pagead/interaction^
            ||static.doubleclick.net/instream/ad_status.js^
            ||2mdn.net/instream^
            ||tpc.googlesyndication.com^
            ||pagead2.googlesyndication.com^
            ||gstatic.com/cast/sdk/libs/ads^
            ||imasdk.googleapis.com^
            ||youtube.com/error_204^
            ||youtube.com/csi_204^
            ||youtube.com/generate_204^
            ||youtube.com/api/stats/watchtime^
            ||youtube.com/api/stats/delayplay^
            ||youtube.com/api/stats/playback^
            ||youtube.com/pcs/activeview^
            ||youtube.com/pagead/paralleladview^
            ||youtube.com/pagead/viewthroughconversion^
            
            # Mobile app ads
            */ads/*
            */adsdk/*
            */advertise/*
            */advertisement/*
            */advertising/*
            */adserver/*
            */adservice/*
            */adnetwork/*
            */analytics/*
            */telemetry/*
            */metrics/*
            */tracking/*
            */banner/*
            */popup/*
            */popunder/*
            */interstitial/*
            */sponsorship/*
            */promoted/*
        """
        
        loadFilterRules(defaultRules)
    }
}