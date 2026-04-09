import Foundation

enum StatusBarBridgeTab: String, Codable, CaseIterable, Hashable {
    case overview
    case codex
    case gemini

    var debugValue: Int32 {
        switch self {
        case .overview:
            return 0
        case .codex:
            return 1
        case .gemini:
            return 2
        }
    }

    var displayTitle: String {
        switch self {
        case .overview:
            return "Overview"
        case .codex:
            return "Codex"
        case .gemini:
            return "Gemini"
        }
    }
}

struct StatusBarBridgeMetric: Codable, Identifiable, Hashable {
    let title: String
    let percent: UInt8
    let leftText: String
    let resetText: String

    var id: String {
        "\(title)-\(resetText)-\(percent)"
    }
}

struct StatusBarBridgeSection: Codable, Identifiable, Hashable {
    let id: String
    let providerId: String
    let providerTitle: String
    let email: String
    let subtitle: String
    let plan: String?
    let isActive: Bool
    let needsRelogin: Bool
    let metrics: [StatusBarBridgeMetric]
    let switchAccountId: String?
}

struct StatusBarBridgePayload: Codable, Hashable {
    let selectedTab: StatusBarBridgeTab
    let sections: [StatusBarBridgeSection]

    static let empty = StatusBarBridgePayload(selectedTab: .overview, sections: [])

    init(selectedTab: StatusBarBridgeTab, sections: [StatusBarBridgeSection]) {
        self.selectedTab = selectedTab
        self.sections = sections
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let selectedTabRaw = (try? container.decode(String.self, forKey: .selectedTab)) ?? ""
        selectedTab = StatusBarBridgeTab(rawValue: selectedTabRaw) ?? .overview
        sections = (try? container.decode([StatusBarBridgeSection].self, forKey: .sections)) ?? []
    }

    private enum CodingKeys: String, CodingKey {
        case selectedTab
        case sections
    }

    static func decode(json: UnsafePointer<CChar>?) -> StatusBarBridgePayload? {
        guard let json else {
            return nil
        }

        return decode(jsonString: String(cString: json))
    }

    static func decode(jsonString: String) -> StatusBarBridgePayload? {
        guard let data = jsonString.data(using: .utf8) else {
            return nil
        }

        return try? JSONDecoder().decode(StatusBarBridgePayload.self, from: data)
    }

    func selectingTab(_ tab: StatusBarBridgeTab) -> StatusBarBridgePayload {
        StatusBarBridgePayload(selectedTab: tab, sections: sections)
    }
}

enum StatusBarBridgeAction {
    case selectTab(StatusBarBridgeTab)
    case switchAccount(provider: String, accountId: String)
    case refresh
    case openMainWindow
    case quit

    var jsonString: String? {
        let payload: [String: Any]

        switch self {
        case .selectTab(let tab):
            payload = ["type": "select_tab", "tab": tab.rawValue]
        case .switchAccount(let provider, let accountId):
            payload = [
                "type": "switch_account",
                "provider": provider,
                "account_id": accountId,
            ]
        case .refresh:
            payload = ["type": "refresh"]
        case .openMainWindow:
            payload = ["type": "open_main_window"]
        case .quit:
            payload = ["type": "quit"]
        }

        guard let data = try? JSONSerialization.data(withJSONObject: payload),
              let jsonString = String(data: data, encoding: .utf8) else {
            return nil
        }

        return jsonString
    }
}
