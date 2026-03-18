import Foundation

enum EpisodeLoader {
    static func loadRecent(from logsURL: URL) async -> [Episode] {
        let url = logsURL.appending(path: "recent_episodes.json")
        guard let data = try? Data(contentsOf: url) else { return [] }

        // Try as array first, then as wrapped object
        if let episodes = try? JSONDecoder().decode([Episode].self, from: data) {
            return episodes
        }
        if let wrapped = try? JSONDecoder().decode(TopEpisodesFile.self, from: data) {
            return wrapped.episodes
        }
        return []
    }

    static func loadTop(from logsURL: URL) async -> [Episode] {
        let url = logsURL.appending(path: "top_episodes.json")
        guard let data = try? Data(contentsOf: url) else { return [] }

        if let wrapped = try? JSONDecoder().decode(TopEpisodesFile.self, from: data) {
            return wrapped.episodes
        }
        if let episodes = try? JSONDecoder().decode([Episode].self, from: data) {
            return episodes
        }
        return []
    }
}
