import Foundation

struct StatusBarMenuPresentation {
    struct AccountSection: Identifiable, Hashable {
        let id: String
        let section: StatusBarBridgeSection
        let metrics: [StatusBarBridgeMetric]
    }

    static let visibleTabs: [StatusBarBridgeTab] = [.codex, .claude, .gemini]
    static let footerActionCount: Int32 = 3

    let rawSelectedTab: StatusBarBridgeTab
    let accountSections: [AccountSection]

    init(payload: StatusBarBridgePayload) {
        self.rawSelectedTab = payload.selectedTab
        self.accountSections = payload.sections
            .enumerated()
            .sorted { lhs, rhs in
                if lhs.element.isActive != rhs.element.isActive {
                    return lhs.element.isActive && !rhs.element.isActive
                }

                let leftQuota = Self.primaryQuotaPercent(for: lhs.element)
                let rightQuota = Self.primaryQuotaPercent(for: rhs.element)
                if leftQuota != rightQuota {
                    if let leftQuota, let rightQuota {
                        return leftQuota > rightQuota
                    }

                    return leftQuota != nil
                }

                return lhs.offset < rhs.offset
            }
            .map { indexedSection in
                let section = indexedSection.element
                return AccountSection(id: section.id, section: section, metrics: section.metrics)
            }
    }

    static func primaryQuotaPercent(for section: StatusBarBridgeSection) -> UInt8? {
        if section.needsRelogin {
            return nil
        }

        return section.primaryQuotaPercent
    }

    var selectedTab: StatusBarBridgeTab {
        if rawSelectedTab != .overview {
            return rawSelectedTab
        }

        if let providerID = accountSections.first?.section.providerId,
           let providerTab = StatusBarBridgeTab(rawValue: providerID) {
            return providerTab
        }

        return .codex
    }

    var activeSectionIndex: Int32 {
        Int32(self.accountSections.firstIndex(where: { $0.section.isActive }) ?? -1)
    }

    var totalMetricCount: Int32 {
        Int32(self.accountSections.reduce(0) { partialResult, accountSection in
            partialResult + accountSection.metrics.count
        })
    }

    var activeSection: AccountSection? {
        self.accountSections.first(where: { $0.section.isActive }) ?? self.accountSections.first
    }

    var currentDetailSection: AccountSection? {
        self.visibleDetailSections.first(where: { $0.section.isActive })
    }

    var scrollableDetailSections: [AccountSection] {
        guard let currentDetailSection else {
            return self.visibleDetailSections
        }

        return self.visibleDetailSections.filter { $0.id != currentDetailSection.id }
    }

    var accountChipSections: [AccountSection] {
        []
    }

    var showsAccountChips: Bool {
        false
    }

    var accountChipLayoutAxisDebugValue: Int32 {
        0
    }

    var visibleDetailSectionCount: Int32 {
        Int32(self.visibleDetailSections.count)
    }

    var pinnedDetailSectionCount: Int32 {
        self.currentDetailSection == nil ? 0 : 1
    }

    var scrollableDetailSectionCount: Int32 {
        Int32(self.scrollableDetailSections.count)
    }

    var visibleDetailSections: [AccountSection] {
        let filtered = self.accountSections.filter { $0.section.providerId == self.selectedTab.rawValue }
        return filtered.isEmpty ? self.accountSections : filtered
    }
}
