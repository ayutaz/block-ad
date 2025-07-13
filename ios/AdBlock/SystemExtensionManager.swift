#if os(macOS)
import Foundation
import SystemExtensions

class SystemExtensionManager: NSObject {
    
    static let shared = SystemExtensionManager()
    
    private var extensionBundleIdentifier = "com.adblock.app.networkextension"
    private var activationRequest: OSSystemExtensionRequest?
    
    private override init() {
        super.init()
    }
    
    func installSystemExtension(completion: @escaping (Result<Void, Error>) -> Void) {
        guard activationRequest == nil else {
            completion(.failure(SystemExtensionError.requestInProgress))
            return
        }
        
        let request = OSSystemExtensionRequest.activationRequest(
            forExtensionWithIdentifier: extensionBundleIdentifier,
            queue: .main
        )
        
        request.delegate = self
        
        activationRequest = request
        OSSystemExtensionManager.shared.submitRequest(request)
        
        // Store completion handler
        self.completionHandler = completion
    }
    
    func removeSystemExtension(completion: @escaping (Result<Void, Error>) -> Void) {
        let request = OSSystemExtensionRequest.deactivationRequest(
            forExtensionWithIdentifier: extensionBundleIdentifier,
            queue: .main
        )
        
        request.delegate = self
        
        OSSystemExtensionManager.shared.submitRequest(request)
        
        self.completionHandler = completion
    }
    
    private var completionHandler: ((Result<Void, Error>) -> Void)?
}

// MARK: - OSSystemExtensionRequestDelegate

extension SystemExtensionManager: OSSystemExtensionRequestDelegate {
    
    func request(_ request: OSSystemExtensionRequest, actionForReplacingExtension existing: OSSystemExtensionProperties, withExtension ext: OSSystemExtensionProperties) -> OSSystemExtensionRequest.ReplacementAction {
        // Always replace with the new version
        return .replace
    }
    
    func requestNeedsUserApproval(_ request: OSSystemExtensionRequest) {
        print("System Extension needs user approval")
        // The user needs to approve the extension in System Preferences
    }
    
    func request(_ request: OSSystemExtensionRequest, didFinishWithResult result: OSSystemExtensionRequest.Result) {
        print("System Extension request finished with result: \(result.rawValue)")
        
        switch result {
        case .completed:
            completionHandler?(.success(()))
        case .willCompleteAfterReboot:
            completionHandler?(.failure(SystemExtensionError.rebootRequired))
        @unknown default:
            completionHandler?(.failure(SystemExtensionError.unknown))
        }
        
        activationRequest = nil
        completionHandler = nil
    }
    
    func request(_ request: OSSystemExtensionRequest, didFailWithError error: Error) {
        print("System Extension request failed: \(error)")
        
        completionHandler?(.failure(error))
        activationRequest = nil
        completionHandler = nil
    }
}

// MARK: - Errors

enum SystemExtensionError: LocalizedError {
    case requestInProgress
    case rebootRequired
    case unknown
    
    var errorDescription: String? {
        switch self {
        case .requestInProgress:
            return "System Extension のインストールが既に進行中です"
        case .rebootRequired:
            return "System Extension のインストールを完了するには再起動が必要です"
        case .unknown:
            return "System Extension のインストール中に不明なエラーが発生しました"
        }
    }
}

#endif