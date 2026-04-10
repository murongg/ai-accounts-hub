import AppKit
import SwiftUI

enum StatusBarPanelTokens {
    static let panelWidth: CGFloat = 376
    static let panelInset: CGFloat = 5
    static let panelContentPadding: CGFloat = 8
    static let switcherHeight: CGFloat = 28
    static let accountChipHeight: CGFloat = 24
    static let hoverCornerRadius: CGFloat = 8
    static let sectionSpacing: CGFloat = 8
    static let rowSpacing: CGFloat = 5
    static let compactSpacing: CGFloat = 3
    static let sectionBoundarySpacing: CGFloat = 2
    static let itemSpacing: CGFloat = 8
    static let switcherContainerPadding: CGFloat = 2
    static let switcherItemSpacing: CGFloat = 4
    static let switcherLabelSpacing: CGFloat = 6
    static let interactiveRowHorizontalPadding: CGFloat = 8
    static let interactiveRowVerticalPadding: CGFloat = 3
    static let accountSectionHoverHorizontalPadding: CGFloat = 8
    static let accountSectionHoverVerticalPadding: CGFloat = 6
    static let panelCornerRadius: CGFloat = 14
    static let progressHeight: CGFloat = 5
    static let footerActionHorizontalPadding: CGFloat = 8
    static let footerActionVerticalPadding: CGFloat = 1
    static let actionRowHeight: CGFloat = 24
    static let actionIconColumnWidth: CGFloat = 14
    static let actionChevronColumnWidth: CGFloat = 10
}

enum StatusBarPanelPalette {
    private static let textPrimaryColor = NSColor.labelColor
    private static let textSecondaryColor = NSColor.secondaryLabelColor

    static let panelTint = Color(nsColor: NSColor(white: 1.0, alpha: 0.12))
    static let panelOverlay = Color(nsColor: NSColor(white: 1.0, alpha: 0.18))
    static let border = Color(nsColor: NSColor(white: 1.0, alpha: 0.35))
    static let separator = Color(nsColor: NSColor(white: 0.45, alpha: 0.26))
    static let chipIdle = Color(nsColor: NSColor(white: 0.92, alpha: 0.88))
    static let rowHover = Color(nsColor: NSColor(red: 0.862, green: 0.899, blue: 0.980, alpha: 0.72))
    static let progressTrack = Color(nsColor: NSColor(white: 0.78, alpha: 0.9))
    static let textPrimary = Color(nsColor: textPrimaryColor)
    static let textSecondary = Color(nsColor: textSecondaryColor)
    static let codexAccent = Color(
        nsColor: NSColor(red: 0.109, green: 0.522, blue: 0.988, alpha: 1.0)
    )
    static let claudeAccent = Color(
        nsColor: NSColor(red: 0.851, green: 0.467, blue: 0.341, alpha: 1.0)
    )
    static let geminiAccent = Color(
        nsColor: NSColor(red: 0.675, green: 0.514, blue: 0.973, alpha: 1.0)
    )
    static let overviewAccent = Color(
        nsColor: NSColor(red: 0.443, green: 0.443, blue: 0.486, alpha: 1.0)
    )

    static func debugTextPaletteAdaptsToAppearance() -> Bool {
        let lightPrimary = resolvedLuminance(of: textPrimaryColor, appearanceName: .aqua)
        let darkPrimary = resolvedLuminance(of: textPrimaryColor, appearanceName: .darkAqua)
        let lightSecondary = resolvedLuminance(of: textSecondaryColor, appearanceName: .aqua)
        let darkSecondary = resolvedLuminance(of: textSecondaryColor, appearanceName: .darkAqua)

        return lightPrimary < 0.35
            && darkPrimary > 0.75
            && lightSecondary < 0.55
            && darkSecondary > 0.55
            && darkPrimary > lightPrimary
            && darkSecondary > lightSecondary
    }

    private static func resolvedLuminance(of color: NSColor, appearanceName: NSAppearance.Name) -> CGFloat {
        var resolved = color

        if let appearance = NSAppearance(named: appearanceName) {
            appearance.performAsCurrentDrawingAppearance {
                resolved = color.usingColorSpace(NSColorSpace.sRGB) ?? color
            }
        }

        return (0.2126 * resolved.redComponent)
            + (0.7152 * resolved.greenComponent)
            + (0.0722 * resolved.blueComponent)
    }
}

enum StatusBarPanelAccent {
    static func color(for tab: StatusBarBridgeTab) -> Color {
        switch tab {
        case .overview:
            return StatusBarPanelPalette.overviewAccent
        case .codex:
            return StatusBarPanelPalette.codexAccent
        case .claude:
            return StatusBarPanelPalette.claudeAccent
        case .gemini:
            return StatusBarPanelPalette.geminiAccent
        }
    }

    static func color(for providerID: String) -> Color {
        switch providerID {
        case "codex":
            return StatusBarPanelPalette.codexAccent
        case "claude":
            return StatusBarPanelPalette.claudeAccent
        case "gemini":
            return StatusBarPanelPalette.geminiAccent
        default:
            return StatusBarPanelPalette.overviewAccent
        }
    }
}

enum StatusBarSizing {
    static let panelWidth = StatusBarPanelTokens.panelWidth
    static let panelInset = StatusBarPanelTokens.panelInset
}
