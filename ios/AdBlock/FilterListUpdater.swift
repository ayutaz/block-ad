import Foundation

/// Manages filter list updates from remote sources
public class FilterListUpdater {
    
    private let defaultFilterURLs = [
        "https://easylist.to/easylist/easylist.txt",
        "https://raw.githubusercontent.com/easylist/easylist/master/easylist/easylist.txt",
        "https://secure.fanboy.co.nz/fanboy-complete.txt"
    ]
    
    private let session = URLSession.shared
    private let engine: AdBlockEngine
    
    public init(engine: AdBlockEngine) {
        self.engine = engine
    }
    
    /// Update filter lists from remote sources
    public func updateFilterLists(completion: @escaping (Result<String, Error>) -> Void) {
        Task {
            do {
                let filterContent = try await downloadFilterLists()
                let success = engine.loadFilterList(filterContent)
                
                if success {
                    // Save to UserDefaults for offline use
                    UserDefaults.standard.set(filterContent, forKey: "cached_filter_lists")
                    UserDefaults.standard.set(Date(), forKey: "last_filter_update")
                    
                    completion(.success("フィルターリストを更新しました"))
                } else {
                    completion(.failure(FilterListError.loadFailed))
                }
            } catch {
                completion(.failure(error))
            }
        }
    }
    
    /// Load cached filter lists
    public func loadCachedFilters() -> Bool {
        guard let cached = UserDefaults.standard.string(forKey: "cached_filter_lists") else {
            return false
        }
        return engine.loadFilterList(cached)
    }
    
    /// Check if filter lists need update
    public func needsUpdate() -> Bool {
        guard let lastUpdate = UserDefaults.standard.object(forKey: "last_filter_update") as? Date else {
            return true
        }
        
        // Update if older than 7 days
        let daysSinceUpdate = Calendar.current.dateComponents([.day], from: lastUpdate, to: Date()).day ?? 0
        return daysSinceUpdate >= 7
    }
    
    private func downloadFilterLists() async throws -> String {
        var combinedFilters = ""
        
        for url in defaultFilterURLs {
            do {
                let filterContent = try await downloadFromURL(url)
                combinedFilters += filterContent + "\n"
                break // Use first successful download
            } catch {
                // Try next URL
                continue
            }
        }
        
        if combinedFilters.isEmpty {
            throw FilterListError.downloadFailed
        }
        
        return combinedFilters
    }
    
    private func downloadFromURL(_ urlString: String) async throws -> String {
        guard let url = URL(string: urlString) else {
            throw FilterListError.invalidURL
        }
        
        let (data, response) = try await session.data(from: url)
        
        guard let httpResponse = response as? HTTPURLResponse,
              httpResponse.statusCode == 200 else {
            throw FilterListError.httpError
        }
        
        guard let content = String(data: data, encoding: .utf8) else {
            throw FilterListError.decodingError
        }
        
        return content
    }
}

public enum FilterListError: Error {
    case invalidURL
    case downloadFailed
    case httpError
    case decodingError
    case loadFailed
}