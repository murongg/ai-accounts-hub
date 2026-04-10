import SwiftUI

struct StatusBarMenuRootView: View {
    @ObservedObject var session: StatusBarMenuSession
    let secondaryDetailMaxHeight: CGFloat?
    let onAction: (StatusBarBridgeAction) -> Void

    private var presentation: StatusBarMenuPresentation {
        StatusBarMenuPresentation(payload: session.displayedPayload)
    }

    init(
        session: StatusBarMenuSession,
        secondaryDetailMaxHeight: CGFloat? = nil,
        onAction: @escaping (StatusBarBridgeAction) -> Void
    ) {
        self.session = session
        self.secondaryDetailMaxHeight = secondaryDetailMaxHeight
        self.onAction = onAction
    }

    var body: some View {
        VStack(alignment: .leading, spacing: StatusBarPanelTokens.sectionSpacing) {
            StatusBarTopSwitcher(selectedTab: presentation.selectedTab, onAction: onAction)

            StatusBarSectionDivider()

            if presentation.visibleDetailSections.isEmpty {
                StatusBarEmptyState()
            } else {
                VStack(alignment: .leading, spacing: StatusBarPanelTokens.sectionSpacing) {
                    if let currentDetailSection = presentation.currentDetailSection {
                        StatusBarAccountSection(account: currentDetailSection, onAction: onAction)
                    }

                    if !presentation.scrollableDetailSections.isEmpty {
                        if presentation.currentDetailSection != nil {
                            StatusBarPinnedScrollBoundary()
                        }

                        StatusBarScrollableAccountSections(
                            accounts: presentation.scrollableDetailSections,
                            maxHeight: secondaryDetailMaxHeight,
                            onAction: onAction
                        )
                    }
                }
            }

            StatusBarSectionDivider()

            StatusBarFooterRows(onAction: onAction)
        }
        .padding(.horizontal, StatusBarPanelTokens.panelContentPadding)
        .padding(.vertical, StatusBarPanelTokens.panelContentPadding)
        .frame(width: StatusBarPanelTokens.panelWidth, alignment: .topLeading)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
    }
}

extension StatusBarMenuRootView {
    static let usesOuterPanelChrome = false
    static let usesScrollableDetailContainer = true
}

private struct StatusBarPinnedScrollBoundary: View {
    var body: some View {
        VStack(spacing: 0) {
            StatusBarSectionDivider()
            Color.clear.frame(height: StatusBarPanelTokens.sectionBoundarySpacing)
        }
    }
}

private struct StatusBarScrollableAccountSections: View {
    let accounts: [StatusBarMenuPresentation.AccountSection]
    let maxHeight: CGFloat?
    let onAction: (StatusBarBridgeAction) -> Void

    var body: some View {
        Group {
            if let maxHeight {
                ScrollView(.vertical, showsIndicators: true) {
                    StatusBarAccountSectionList(accounts: accounts, onAction: onAction)
                }
                .frame(maxHeight: maxHeight)
            } else {
                StatusBarAccountSectionList(accounts: accounts, onAction: onAction)
            }
        }
    }
}

private struct StatusBarAccountSectionList: View {
    let accounts: [StatusBarMenuPresentation.AccountSection]
    let onAction: (StatusBarBridgeAction) -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: StatusBarPanelTokens.sectionSpacing) {
            ForEach(Array(accounts.enumerated()), id: \.element.id) { index, account in
                StatusBarAccountSection(account: account, onAction: onAction)

                if index < accounts.count - 1 {
                    StatusBarSectionDivider()
                }
            }
        }
    }
}

private struct StatusBarAccountSection: View {
    let account: StatusBarMenuPresentation.AccountSection
    let onAction: (StatusBarBridgeAction) -> Void

    var body: some View {
        if let switchAccountID = account.section.switchAccountId, !switchAccountID.isEmpty {
            Button {
                onAction(.switchAccount(provider: account.section.providerId, accountId: switchAccountID))
            } label: {
                StatusBarHoverSurface(
                    horizontalPadding: StatusBarPanelTokens.accountSectionHoverHorizontalPadding,
                    verticalPadding: StatusBarPanelTokens.accountSectionHoverVerticalPadding
                ) {
                    StatusBarDetailSection(section: account.section)
                }
            }
            .buttonStyle(.plain)
        } else {
            StatusBarDetailSection(section: account.section)
        }
    }
}

private struct StatusBarDetailSection: View {
    let section: StatusBarBridgeSection

    var body: some View {
        VStack(alignment: .leading, spacing: StatusBarPanelTokens.sectionSpacing) {
            StatusBarDetailHeader(section: section)

            if section.metrics.isEmpty {
                Text(section.needsRelogin ? "Re-login required before refresh." : "No usage metrics available yet.")
                    .font(.system(size: 11, weight: .medium))
                    .foregroundStyle(StatusBarPanelPalette.textSecondary)
            } else {
                VStack(alignment: .leading, spacing: StatusBarPanelTokens.sectionSpacing) {
                    ForEach(section.metrics) { metric in
                        StatusBarMetricRow(metric: metric, providerID: section.providerId)
                    }
                }
            }
        }
    }
}

private struct StatusBarDetailHeader: View {
    let section: StatusBarBridgeSection

    var body: some View {
        VStack(alignment: .leading, spacing: StatusBarPanelTokens.compactSpacing) {
            HStack(alignment: .firstTextBaseline, spacing: StatusBarPanelTokens.itemSpacing) {
                Text(section.providerTitle)
                    .font(.system(size: 14, weight: .bold))
                    .foregroundStyle(StatusBarPanelPalette.textPrimary)

                Spacer(minLength: StatusBarPanelTokens.itemSpacing)

                Text(section.email)
                    .font(.system(size: 12, weight: .medium))
                    .foregroundStyle(StatusBarPanelPalette.textSecondary)
                    .lineLimit(1)
                    .truncationMode(.middle)
                    .allowsTightening(true)
                    .layoutPriority(1)
            }

            HStack(alignment: .firstTextBaseline, spacing: StatusBarPanelTokens.itemSpacing) {
                Text(section.subtitle)
                    .font(.system(size: 11, weight: .medium))
                    .foregroundStyle(StatusBarPanelPalette.textSecondary)

                Spacer(minLength: StatusBarPanelTokens.itemSpacing)

                HStack(spacing: 6) {
                    if section.isActive {
                        Text("Current")
                            .font(.system(size: 10, weight: .semibold))
                            .foregroundStyle(StatusBarPanelAccent.color(for: section.providerId))
                    }

                    if let plan = section.plan, !plan.isEmpty {
                        Text(plan)
                            .font(.system(size: 11, weight: .semibold))
                            .foregroundStyle(StatusBarPanelPalette.textSecondary)
                    }
                }
            }
        }
    }
}

private struct StatusBarMetricRow: View {
    let metric: StatusBarBridgeMetric
    let providerID: String

    private var progressTint: Color {
        StatusBarQuotaTone(percent: metric.percent).color(for: providerID)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: StatusBarPanelTokens.rowSpacing) {
            Text(metric.title)
                .font(.system(size: 13, weight: .bold))
                .foregroundStyle(StatusBarPanelPalette.textPrimary)

            StatusBarProgressBar(percent: metric.percent, tint: progressTint)
                .frame(height: StatusBarPanelTokens.progressHeight)

            HStack(spacing: StatusBarPanelTokens.itemSpacing) {
                Text(metric.leftText)
                    .font(.system(size: 11, weight: .semibold))
                    .foregroundStyle(StatusBarPanelPalette.textPrimary)

                Text(metric.resetText)
                    .font(.system(size: 11, weight: .semibold))
                    .foregroundStyle(StatusBarPanelPalette.textSecondary)
                    .frame(maxWidth: .infinity, alignment: .trailing)
            }
        }
    }
}

private struct StatusBarProgressBar: View {
    let percent: UInt8
    let tint: Color

    var body: some View {
        GeometryReader { proxy in
            ZStack(alignment: .leading) {
                Capsule(style: .continuous)
                    .fill(StatusBarPanelPalette.progressTrack)

                Capsule(style: .continuous)
                    .fill(tint)
                    .frame(width: statusBarProgressFillWidth(totalWidth: proxy.size.width, percent: percent))
            }
        }
    }
}

func statusBarProgressFillWidth(totalWidth: CGFloat, percent: UInt8) -> CGFloat {
    guard totalWidth > 0, percent > 0 else {
        return 0
    }

    let proportionalWidth = totalWidth * CGFloat(percent) / 100.0
    return min(totalWidth, max(8, proportionalWidth))
}

private struct StatusBarFooterRows: View {
    let onAction: (StatusBarBridgeAction) -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            StatusBarActionRow(title: "Refresh", systemImage: "arrow.clockwise", action: .refresh, onAction: onAction)
            StatusBarActionRow(title: "Open App", systemImage: "macwindow", action: .openMainWindow, onAction: onAction)
            StatusBarActionRow(title: "Quit", systemImage: "power", action: .quit, onAction: onAction)
        }
    }
}

private struct StatusBarEmptyState: View {
    var body: some View {
        VStack(alignment: .leading, spacing: StatusBarPanelTokens.rowSpacing) {
            Text("No accounts available")
                .font(.system(size: 12, weight: .semibold))
                .foregroundStyle(StatusBarPanelPalette.textPrimary)

            Text("Add a Codex, Claude, or Gemini account in the main app to populate the menu bar view.")
                .font(.system(size: 10, weight: .medium))
                .foregroundStyle(StatusBarPanelPalette.textSecondary)
        }
        .padding(.horizontal, StatusBarPanelTokens.panelContentPadding)
        .padding(.vertical, StatusBarPanelTokens.panelContentPadding)
    }
}
