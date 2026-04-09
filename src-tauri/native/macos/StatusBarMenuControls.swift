import AppKit
import SwiftUI

enum StatusBarProviderIconProvider {
    enum ResourceVariant: Int32 {
        case vector = 1
        case raster = 2
    }

    static func image(for tab: StatusBarBridgeTab, template: Bool) -> NSImage? {
        image(
            for: tab,
            template: template,
            bundleResourceURL: Bundle.main.resourceURL,
            currentDirectoryURL: URL(fileURLWithPath: FileManager.default.currentDirectoryPath)
        )
    }

    static func image(
        for tab: StatusBarBridgeTab,
        template: Bool,
        bundleResourceURL: URL?,
        currentDirectoryURL: URL
    ) -> NSImage? {
        guard let resource = resource(for: tab),
              let resourceURL = resourceURL(
                  for: resource,
                  bundleResourceURL: bundleResourceURL,
                  currentDirectoryURL: currentDirectoryURL
              ),
              let data = try? Data(contentsOf: resourceURL),
              let loadedImage = NSImage(data: data),
              let image = loadedImage.copy() as? NSImage else {
            return nil
        }

        image.size = NSSize(width: 12, height: 12)
        image.isTemplate = template
        return image
    }

    static func isReady(for tab: StatusBarBridgeTab) -> Bool {
        image(for: tab, template: false) != nil
    }

    static func resourceVariant(for tab: StatusBarBridgeTab) -> ResourceVariant? {
        resource(for: tab)?.variant
    }

    static func resourceVariant(
        for tab: StatusBarBridgeTab,
        bundleResourceURL: URL?,
        currentDirectoryURL: URL
    ) -> ResourceVariant? {
        guard let resource = resource(for: tab),
              resourceURL(
                  for: resource,
                  bundleResourceURL: bundleResourceURL,
                  currentDirectoryURL: currentDirectoryURL
              ) != nil else {
            return nil
        }

        return resource.variant
    }

    private static func resource(for tab: StatusBarBridgeTab) -> (name: String, fileExtension: String, variant: ResourceVariant)? {
        switch tab {
        case .overview:
            return nil
        case .codex:
            return ("openai", "svg", .vector)
        case .gemini:
            return ("gemini", "png", .raster)
        }
    }

    private static func resourceURL(
        for resource: (name: String, fileExtension: String, variant: ResourceVariant),
        bundleResourceURL: URL?,
        currentDirectoryURL: URL
    ) -> URL? {
        let candidates: [URL?]

        switch resource.variant {
        case .vector:
            candidates = [
                Bundle.main.url(forResource: resource.name, withExtension: resource.fileExtension),
                bundleResourceURL?.appendingPathComponent("\(resource.name).\(resource.fileExtension)"),
                bundleResourceURL?.appendingPathComponent("_up_/src/assets/\(resource.name).\(resource.fileExtension)"),
                bundleResourceURL?.appendingPathComponent("src/assets/\(resource.name).\(resource.fileExtension)"),
                currentDirectoryURL.appendingPathComponent("src/assets/\(resource.name).\(resource.fileExtension)"),
                currentDirectoryURL.appendingPathComponent("../src/assets/\(resource.name).\(resource.fileExtension)"),
            ]
        case .raster:
            candidates = [
                Bundle.main.url(forResource: resource.name, withExtension: resource.fileExtension),
                bundleResourceURL?.appendingPathComponent("\(resource.name).\(resource.fileExtension)"),
                bundleResourceURL?.appendingPathComponent("native/macos/provider-icons/\(resource.name).\(resource.fileExtension)"),
                currentDirectoryURL.appendingPathComponent("src-tauri/native/macos/provider-icons/\(resource.name).\(resource.fileExtension)"),
                currentDirectoryURL.appendingPathComponent("native/macos/provider-icons/\(resource.name).\(resource.fileExtension)"),
            ]
        }

        return candidates
            .compactMap { $0 }
            .first(where: { FileManager.default.fileExists(atPath: $0.path) })
    }
}

struct StatusBarMaterialBackground: NSViewRepresentable {
    func makeNSView(context: Context) -> NSVisualEffectView {
        let view = NSVisualEffectView()
        view.material = .menu
        view.blendingMode = .withinWindow
        view.state = .followsWindowActiveState
        return view
    }

    func updateNSView(_ nsView: NSVisualEffectView, context: Context) {}
}

struct StatusBarSectionDivider: View {
    var body: some View {
        Rectangle()
            .fill(StatusBarPanelPalette.separator)
            .frame(height: 1)
    }
}

struct StatusBarHoverSurface<Content: View>: View {
    let isSelected: Bool
    let selectedFill: Color?
    let horizontalPadding: CGFloat
    let verticalPadding: CGFloat
    @ViewBuilder let content: () -> Content

    @State private var isHovered = false

    init(
        isSelected: Bool = false,
        selectedFill: Color? = nil,
        horizontalPadding: CGFloat = StatusBarPanelTokens.interactiveRowHorizontalPadding,
        verticalPadding: CGFloat = StatusBarPanelTokens.interactiveRowVerticalPadding,
        @ViewBuilder content: @escaping () -> Content
    ) {
        self.isSelected = isSelected
        self.selectedFill = selectedFill
        self.horizontalPadding = horizontalPadding
        self.verticalPadding = verticalPadding
        self.content = content
    }

    var body: some View {
        content()
            .padding(.horizontal, horizontalPadding)
            .padding(.vertical, verticalPadding)
            .background(
                RoundedRectangle(cornerRadius: StatusBarPanelTokens.hoverCornerRadius, style: .continuous)
                    .fill(backgroundColor)
            )
            .animation(.easeOut(duration: 0.12), value: isHovered)
            .onHover { hovering in
                isHovered = hovering
            }
    }

    private var backgroundColor: Color {
        if isSelected {
            return selectedFill ?? StatusBarPanelPalette.rowHover.opacity(0.95)
        }

        if isHovered {
            return StatusBarPanelPalette.rowHover.opacity(0.78)
        }

        return .clear
    }
}

struct StatusBarTopSwitcher: View {
    let selectedTab: StatusBarBridgeTab
    let onAction: (StatusBarBridgeAction) -> Void

    var body: some View {
        HStack(spacing: StatusBarPanelTokens.switcherItemSpacing) {
            ForEach(StatusBarMenuPresentation.visibleTabs, id: \.self) { tab in
                Button {
                    onAction(.selectTab(tab))
                } label: {
                    StatusBarHoverSurface(
                        isSelected: selectedTab == tab,
                        selectedFill: StatusBarPanelAccent.color(for: tab)
                    ) {
                        HStack(spacing: StatusBarPanelTokens.switcherLabelSpacing) {
                            StatusBarTopSwitcherIcon(tab: tab, isSelected: selectedTab == tab)

                            Text(tab.displayTitle)
                                .font(.system(size: 11, weight: selectedTab == tab ? .semibold : .medium))
                                .foregroundStyle(
                                    selectedTab == tab
                                        ? Color.white
                                        : StatusBarPanelPalette.textSecondary
                                )
                        }
                        .frame(maxWidth: .infinity, minHeight: StatusBarPanelTokens.switcherHeight, alignment: .center)
                    }
                }
                .buttonStyle(.plain)
            }
        }
        .padding(StatusBarPanelTokens.switcherContainerPadding)
        .background(
            RoundedRectangle(cornerRadius: StatusBarPanelTokens.hoverCornerRadius + 2, style: .continuous)
                .fill(StatusBarPanelPalette.panelOverlay)
        )
    }
}

private struct StatusBarTopSwitcherIcon: View {
    let tab: StatusBarBridgeTab
    let isSelected: Bool

    var body: some View {
        if let image = StatusBarProviderIconProvider.image(for: tab, template: usesTemplateRendering) {
            Image(nsImage: image)
                .renderingMode(usesTemplateRendering ? .template : .original)
                .resizable()
                .aspectRatio(contentMode: .fit)
                .frame(width: 12, height: 12)
                .foregroundStyle(iconForeground)
        } else {
            Image(systemName: fallbackSystemIcon)
                .font(.system(size: 10, weight: .semibold))
                .foregroundStyle(iconForeground)
        }
    }

    private var usesTemplateRendering: Bool {
        switch tab {
        case .overview:
            return true
        case .codex:
            return true
        case .gemini:
            return false
        }
    }

    private var iconForeground: Color {
        isSelected ? .white : StatusBarPanelPalette.textSecondary
    }

    private var fallbackSystemIcon: String {
        switch tab {
        case .overview:
            return "square.grid.2x2"
        case .codex:
            return "swirl.circle.righthalf.filled"
        case .gemini:
            return "sparkles"
        }
    }
}

struct StatusBarAccountChipRow: View {
    let sections: [StatusBarMenuPresentation.AccountSection]
    let activeSectionID: String?
    let onAction: (StatusBarBridgeAction) -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: StatusBarPanelTokens.compactSpacing) {
            ForEach(sections) { account in
                chip(for: account)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }

    @ViewBuilder
    private func chip(for account: StatusBarMenuPresentation.AccountSection) -> some View {
        let isSelected = account.id == activeSectionID
        let chipBody = StatusBarHoverSurface(
            isSelected: isSelected,
            selectedFill: StatusBarPanelAccent.color(for: account.section.providerId)
        ) {
            HStack(spacing: StatusBarPanelTokens.itemSpacing) {
                Text(account.section.email)
                    .font(.system(size: 11, weight: isSelected ? .semibold : .medium))
                    .foregroundStyle(isSelected ? Color.white : StatusBarPanelPalette.textSecondary)
                    .lineLimit(1)

                Spacer(minLength: StatusBarPanelTokens.itemSpacing)

                if account.section.isActive {
                    Text("Current")
                        .font(.system(size: 10, weight: .semibold))
                        .foregroundStyle(isSelected ? Color.white.opacity(0.92) : StatusBarPanelAccent.color(for: account.section.providerId))
                }
            }
            .frame(maxWidth: .infinity, minHeight: StatusBarPanelTokens.accountChipHeight, alignment: .leading)
        }

        if let switchAccountID = account.section.switchAccountId, !switchAccountID.isEmpty {
            Button {
                onAction(.switchAccount(provider: account.section.providerId, accountId: switchAccountID))
            } label: {
                chipBody
            }
            .buttonStyle(.plain)
            .frame(maxWidth: .infinity, alignment: .leading)
        } else {
            chipBody
                .frame(maxWidth: .infinity, alignment: .leading)
        }
    }
}

struct StatusBarActionRow: View {
    let title: String
    let systemImage: String
    let action: StatusBarBridgeAction
    let onAction: (StatusBarBridgeAction) -> Void

    var body: some View {
        Button {
            onAction(action)
        } label: {
            StatusBarHoverSurface(
                horizontalPadding: StatusBarPanelTokens.footerActionHorizontalPadding,
                verticalPadding: StatusBarPanelTokens.footerActionVerticalPadding
            ) {
                HStack(spacing: StatusBarPanelTokens.itemSpacing) {
                    Image(systemName: systemImage)
                        .font(.system(size: 10, weight: .semibold))
                        .foregroundStyle(StatusBarPanelPalette.textSecondary)
                        .frame(width: StatusBarPanelTokens.actionIconColumnWidth, alignment: .center)

                    Text(title)
                        .font(.system(size: 11, weight: .medium))
                        .foregroundStyle(StatusBarPanelPalette.textPrimary)
                        .frame(maxWidth: .infinity, alignment: .leading)

                    Image(systemName: "chevron.right")
                        .font(.system(size: 9, weight: .semibold))
                        .foregroundStyle(StatusBarPanelPalette.textSecondary.opacity(0.75))
                        .frame(width: StatusBarPanelTokens.actionChevronColumnWidth, alignment: .trailing)
                }
                .frame(maxWidth: .infinity, minHeight: StatusBarPanelTokens.actionRowHeight, alignment: .leading)
            }
        }
        .buttonStyle(.plain)
    }
}
