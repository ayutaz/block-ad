import Foundation

/// Enhanced packet parser for extracting host information
public class PacketParser {
    
    /// Extract host information from a packet
    public static func extractHost(from packet: Data) -> String? {
        guard packet.count >= 20 else { return nil }
        
        return packet.withUnsafeBytes { buffer in
            // Check IP version
            let ipVersion = (buffer[0] >> 4) & 0xF
            
            if ipVersion == 4 {
                return parseIPv4Packet(buffer: buffer, packet: packet)
            } else if ipVersion == 6 {
                return parseIPv6Packet(buffer: buffer, packet: packet)
            }
            
            return nil
        }
    }
    
    private static func parseIPv4Packet(buffer: UnsafeRawBufferPointer, packet: Data) -> String? {
        // Get protocol (6=TCP, 17=UDP)
        let proto = buffer[9]
        guard proto == 6 else { return nil } // Only TCP for TLS
        
        // Get IP header length
        let ipHeaderLength = Int(buffer[0] & 0xF) * 4
        guard packet.count >= ipHeaderLength + 20 else { return nil } // Need TCP header
        
        // Get TCP header
        let tcpHeaderStart = ipHeaderLength
        guard packet.count > tcpHeaderStart + 13 else { return nil }
        
        // Get destination port
        let destPort = (UInt16(buffer[tcpHeaderStart + 2]) << 8) | UInt16(buffer[tcpHeaderStart + 3])
        
        // Only process HTTPS traffic (port 443)
        guard destPort == 443 else { return nil }
        
        // Get TCP header length
        let tcpHeaderLength = (Int(buffer[tcpHeaderStart + 12]) >> 4) * 4
        let payloadStart = ipHeaderLength + tcpHeaderLength
        
        // Extract SNI from TLS handshake if present
        if let sni = extractSNI(from: packet, offset: payloadStart) {
            return sni
        }
        
        // Fall back to IP address
        let destIP = Data(bytes: buffer.baseAddress!.advanced(by: 16), count: 4)
        return destIP.map { String($0) }.joined(separator: ".")
    }
    
    private static func parseIPv6Packet(buffer: UnsafeRawBufferPointer, packet: Data) -> String? {
        // IPv6 header is 40 bytes
        guard packet.count >= 40 else { return nil }
        
        // Get next header (protocol)
        let nextHeader = buffer[6]
        guard nextHeader == 6 else { return nil } // TCP
        
        // Get destination port from TCP header
        guard packet.count >= 40 + 4 else { return nil }
        let destPort = (UInt16(buffer[40 + 2]) << 8) | UInt16(buffer[40 + 3])
        
        // Only process HTTPS traffic
        guard destPort == 443 else { return nil }
        
        // Get TCP header length
        guard packet.count > 40 + 12 else { return nil }
        let tcpHeaderLength = (Int(buffer[40 + 12]) >> 4) * 4
        let payloadStart = 40 + tcpHeaderLength
        
        // Extract SNI from TLS handshake if present
        if let sni = extractSNI(from: packet, offset: payloadStart) {
            return sni
        }
        
        return nil
    }
    
    private static func extractSNI(from packet: Data, offset: Int) -> String? {
        let payloadLength = packet.count - offset
        guard payloadLength >= 5 else { return nil }
        
        return packet.withUnsafeBytes { buffer in
            guard let baseAddress = buffer.baseAddress else { return nil }
            let payload = baseAddress.advanced(by: offset).assumingMemoryBound(to: UInt8.self)
            
            // Check for TLS handshake (0x16)
            guard payload[0] == 0x16 else { return nil }
            
            // Check TLS version (0x0301 = TLS 1.0, 0x0303 = TLS 1.2)
            guard payload[1] == 0x03 && (payload[2] == 0x01 || payload[2] == 0x03) else { return nil }
            
            // Get handshake length
            let handshakeLength = (Int(payload[3]) << 8) | Int(payload[4])
            guard payloadLength >= 5 + handshakeLength else { return nil }
            
            // Check for Client Hello (0x01)
            guard payload[5] == 0x01 else { return nil }
            
            // Skip to extensions
            var pos = 5 + 1 + 3 + 2 + 32 // handshake type + length + version + random
            
            // Skip session ID
            guard pos < payloadLength else { return nil }
            let sessionIdLength = Int(payload[pos])
            pos += 1 + sessionIdLength
            
            // Skip cipher suites
            guard pos + 2 < payloadLength else { return nil }
            let cipherSuitesLength = (Int(payload[pos]) << 8) | Int(payload[pos + 1])
            pos += 2 + cipherSuitesLength
            
            // Skip compression methods
            guard pos < payloadLength else { return nil }
            let compressionLength = Int(payload[pos])
            pos += 1 + compressionLength
            
            // Parse extensions
            guard pos + 2 < payloadLength else { return nil }
            let extensionsLength = (Int(payload[pos]) << 8) | Int(payload[pos + 1])
            pos += 2
            
            let extensionsEnd = pos + extensionsLength
            
            while pos + 4 < extensionsEnd && pos + 4 < payloadLength {
                let extensionType = (Int(payload[pos]) << 8) | Int(payload[pos + 1])
                let extensionLength = (Int(payload[pos + 2]) << 8) | Int(payload[pos + 3])
                pos += 4
                
                // Server Name Indication extension (0x0000)
                if extensionType == 0x0000 {
                    guard pos + extensionLength <= payloadLength else { return nil }
                    
                    // Skip server name list length (2 bytes)
                    pos += 2
                    
                    // Check server name type (0x00 = hostname)
                    guard pos < payloadLength && payload[pos] == 0x00 else { return nil }
                    pos += 1
                    
                    // Get hostname length
                    guard pos + 2 < payloadLength else { return nil }
                    let hostnameLength = (Int(payload[pos]) << 8) | Int(payload[pos + 1])
                    pos += 2
                    
                    // Extract hostname
                    guard pos + hostnameLength <= payloadLength else { return nil }
                    let hostnameData = Data(bytes: payload.advanced(by: pos), count: hostnameLength)
                    return String(data: hostnameData, encoding: .utf8)
                }
                
                pos += extensionLength
            }
            
            return nil
        }
    }
}