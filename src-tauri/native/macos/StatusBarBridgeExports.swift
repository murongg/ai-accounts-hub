import AppKit
import Foundation

private func decodedPayload(from payloadJSON: UnsafePointer<CChar>?) -> StatusBarBridgePayload {
    StatusBarBridgePayload.decode(json: payloadJSON) ?? .empty
}

private func decodedAction(from actionJSON: UnsafePointer<CChar>?) -> StatusBarBridgeAction? {
    guard let actionJSON else {
        return nil
    }

    let jsonString = String(cString: actionJSON)
    guard let data = jsonString.data(using: .utf8),
          let object = try? JSONSerialization.jsonObject(with: data),
          let dictionary = object as? [String: Any],
          let type = dictionary["type"] as? String else {
        return nil
    }

    switch type {
    case "select_tab":
        guard let rawTab = dictionary["tab"] as? String,
              let tab = StatusBarBridgeTab(rawValue: rawTab) else {
            return nil
        }

        return .selectTab(tab)
    case "refresh":
        return .refresh
    case "open_main_window":
        return .openMainWindow
    case "quit":
        return .quit
    case "switch_account":
        guard let provider = dictionary["provider"] as? String,
              let accountID = dictionary["account_id"] as? String else {
            return nil
        }

        return .switchAccount(provider: provider, accountId: accountID)
    default:
        return nil
    }
}

private func decodedJSONObject(from payloadJSON: UnsafePointer<CChar>?) -> [String: Any] {
    guard let payloadJSON else {
        return [:]
    }

    let jsonString = String(cString: payloadJSON)
    guard let data = jsonString.data(using: .utf8),
          let object = try? JSONSerialization.jsonObject(with: data),
          let dictionary = object as? [String: Any] else {
        return [:]
    }

    return dictionary
}

private func decodedURL(from rawPath: UnsafePointer<CChar>?) -> URL? {
    guard let rawPath else {
        return nil
    }

    let path = String(cString: rawPath)
    guard !path.isEmpty else {
        return nil
    }

    return URL(fileURLWithPath: path, isDirectory: true)
}

@_cdecl("aah_status_bar_bridge_swift_initialize")
func aah_status_bar_bridge_swift_initialize(_ callback: AAHStatusBarBridgeCallback?) -> Bool {
    guard let callback else {
        return false
    }

    if Thread.isMainThread {
        return StatusBarBridgeController.shared.initialize(callback: callback)
    }

    var initialized = false
    DispatchQueue.main.sync {
        initialized = StatusBarBridgeController.shared.initialize(callback: callback)
    }
    return initialized
}

@_cdecl("aah_status_bar_bridge_swift_set_payload")
func aah_status_bar_bridge_swift_set_payload(_ payloadJSON: UnsafePointer<CChar>?) {
    guard let payloadJSON else {
        return
    }

    let jsonString = String(cString: payloadJSON)
    guard !jsonString.isEmpty else {
        return
    }

    DispatchQueue.main.async {
        StatusBarBridgeController.shared.updatePayload(jsonString: jsonString)
    }
}

@_cdecl("aah_status_bar_bridge_swift_optional_string_length_from_json")
func aah_status_bar_bridge_swift_optional_string_length_from_json(
    _ payloadJSON: UnsafePointer<CChar>?,
    _ fieldName: UnsafePointer<CChar>?
) -> Int32 {
    guard let fieldName else {
        return 0
    }

    let payload = decodedJSONObject(from: payloadJSON)
    guard let firstSection = (payload["sections"] as? [[String: Any]])?.first else {
        return 0
    }

    switch String(cString: fieldName) {
    case "plan":
        return Int32((firstSection["plan"] as? String)?.count ?? 0)
    case "switchAccountId":
        return Int32((firstSection["switchAccountId"] as? String)?.count ?? 0)
    default:
        return 0
    }
}

@_cdecl("aah_status_bar_bridge_swift_icon_ready")
func aah_status_bar_bridge_swift_icon_ready() -> Int32 {
    guard let image = StatusBarAppIconProvider.load() else {
        return 0
    }

    return image.size.width > 0 && image.size.height > 0 ? 1 : 0
}

@_cdecl("aah_status_bar_bridge_debug_app_icon_source_variant")
func aah_status_bar_bridge_debug_app_icon_source_variant() -> Int32 {
    StatusBarAppIconProvider.sourceVariant()?.rawValue ?? 0
}

@_cdecl("aah_status_bar_bridge_debug_app_icon_source_variant_for_paths")
func aah_status_bar_bridge_debug_app_icon_source_variant_for_paths(
    _ bundleResourcePath: UnsafePointer<CChar>?,
    _ currentDirectoryPath: UnsafePointer<CChar>?
) -> Int32 {
    let currentDirectoryURL = decodedURL(from: currentDirectoryPath)
        ?? URL(fileURLWithPath: FileManager.default.currentDirectoryPath, isDirectory: true)
    return StatusBarAppIconProvider.sourceVariant(
        bundleResourceURL: decodedURL(from: bundleResourcePath),
        currentDirectoryURL: currentDirectoryURL
    )?.rawValue ?? 0
}

@_cdecl("aah_status_bar_bridge_debug_app_icon_is_template")
func aah_status_bar_bridge_debug_app_icon_is_template() -> Int32 {
    (StatusBarAppIconProvider.load()?.isTemplate ?? false) ? 1 : 0
}

@_cdecl("aah_status_bar_bridge_debug_provider_icon_ready_for_tab")
func aah_status_bar_bridge_debug_provider_icon_ready_for_tab(_ tabValue: Int32) -> Int32 {
    let tab: StatusBarBridgeTab?
    switch tabValue {
    case 1:
        tab = .codex
    case 2:
        tab = .claude
    case 3:
        tab = .gemini
    default:
        tab = nil
    }

    guard let tab else {
        return 0
    }

    return StatusBarProviderIconProvider.isReady(for: tab) ? 1 : 0
}

@_cdecl("aah_status_bar_bridge_debug_provider_icon_resource_variant_for_tab")
func aah_status_bar_bridge_debug_provider_icon_resource_variant_for_tab(_ tabValue: Int32) -> Int32 {
    let tab: StatusBarBridgeTab?
    switch tabValue {
    case 1:
        tab = .codex
    case 2:
        tab = .claude
    case 3:
        tab = .gemini
    default:
        tab = nil
    }

    guard let tab,
          let variant = StatusBarProviderIconProvider.resourceVariant(for: tab) else {
        return 0
    }

    return variant.rawValue
}

@_cdecl("aah_status_bar_bridge_debug_provider_icon_resource_variant_for_tab_and_paths")
func aah_status_bar_bridge_debug_provider_icon_resource_variant_for_tab_and_paths(
    _ tabValue: Int32,
    _ bundleResourcePath: UnsafePointer<CChar>?,
    _ currentDirectoryPath: UnsafePointer<CChar>?
) -> Int32 {
    let tab: StatusBarBridgeTab?
    switch tabValue {
    case 1:
        tab = .codex
    case 2:
        tab = .claude
    case 3:
        tab = .gemini
    default:
        tab = nil
    }

    let currentDirectoryURL = decodedURL(from: currentDirectoryPath)
        ?? URL(fileURLWithPath: FileManager.default.currentDirectoryPath, isDirectory: true)

    guard let tab,
          let variant = StatusBarProviderIconProvider.resourceVariant(
              for: tab,
              bundleResourceURL: decodedURL(from: bundleResourcePath),
              currentDirectoryURL: currentDirectoryURL
          ) else {
        return 0
    }

    return variant.rawValue
}

@_cdecl("aah_status_bar_bridge_swift_section_count_from_json")
func aah_status_bar_bridge_swift_section_count_from_json(_ payloadJSON: UnsafePointer<CChar>?) -> Int32 {
    let payload = decodedJSONObject(from: payloadJSON)
    let sections = payload["sections"] as? [[String: Any]] ?? []
    return Int32(sections.count)
}

@_cdecl("aah_status_bar_bridge_swift_selected_tab_value_from_json")
func aah_status_bar_bridge_swift_selected_tab_value_from_json(_ payloadJSON: UnsafePointer<CChar>?) -> Int32 {
    let payload = decodedJSONObject(from: payloadJSON)
    let selectedTab = (payload["selectedTab"] as? String).flatMap(StatusBarBridgeTab.init(rawValue:)) ?? .overview
    return selectedTab.debugValue
}

@_cdecl("aah_status_bar_bridge_debug_visible_tab_count")
func aah_status_bar_bridge_debug_visible_tab_count() -> Int32 {
    Int32(StatusBarMenuPresentation.visibleTabs.count)
}

@_cdecl("aah_status_bar_bridge_debug_text_palette_adapts_to_appearance")
func aah_status_bar_bridge_debug_text_palette_adapts_to_appearance() -> Int32 {
    StatusBarPanelPalette.debugTextPaletteAdaptsToAppearance() ? 1 : 0
}

@_cdecl("aah_status_bar_bridge_debug_hosting_view_matches_requested_system_appearance")
func aah_status_bar_bridge_debug_hosting_view_matches_requested_system_appearance(
    _ prefersDark: Int32
) -> Int32 {
    StatusBarBridgeController.debugHostingViewMatchesRequestedSystemAppearance(prefersDark != 0) ? 1 : 0
}

@_cdecl("aah_status_bar_bridge_debug_active_section_index_from_json")
func aah_status_bar_bridge_debug_active_section_index_from_json(_ payloadJSON: UnsafePointer<CChar>?) -> Int32 {
    let payload = decodedPayload(from: payloadJSON)
    return StatusBarMenuPresentation(payload: payload).activeSectionIndex
}

@_cdecl("aah_status_bar_bridge_debug_total_metric_count_from_json")
func aah_status_bar_bridge_debug_total_metric_count_from_json(_ payloadJSON: UnsafePointer<CChar>?) -> Int32 {
    let payload = decodedPayload(from: payloadJSON)
    return StatusBarMenuPresentation(payload: payload).totalMetricCount
}

@_cdecl("aah_status_bar_bridge_debug_footer_action_count")
func aah_status_bar_bridge_debug_footer_action_count() -> Int32 {
    StatusBarMenuPresentation.footerActionCount
}

@_cdecl("aah_status_bar_bridge_debug_selected_tab_after_action_from_json")
func aah_status_bar_bridge_debug_selected_tab_after_action_from_json(
    _ payloadJSON: UnsafePointer<CChar>?,
    _ actionJSON: UnsafePointer<CChar>?
) -> Int32 {
    let session = StatusBarMenuSession(payload: decodedPayload(from: payloadJSON))

    if let action = decodedAction(from: actionJSON) {
        session.applyOptimistic(action)
    }

    return session.displayedPayload.selectedTab.debugValue
}

@_cdecl("aah_status_bar_bridge_debug_action_keeps_menu_open")
func aah_status_bar_bridge_debug_action_keeps_menu_open(_ actionJSON: UnsafePointer<CChar>?) -> Int32 {
    guard let action = decodedAction(from: actionJSON) else {
        return 0
    }

    return StatusBarBridgeController.debugActionKeepsMenuOpen(action) ? 1 : 0
}

@_cdecl("aah_status_bar_bridge_debug_action_is_handled_locally")
func aah_status_bar_bridge_debug_action_is_handled_locally(_ actionJSON: UnsafePointer<CChar>?) -> Int32 {
    guard let action = decodedAction(from: actionJSON) else {
        return 0
    }

    return StatusBarBridgeController.debugActionIsHandledLocally(action) ? 1 : 0
}

@_cdecl("aah_status_bar_bridge_debug_shows_account_chips_from_json")
func aah_status_bar_bridge_debug_shows_account_chips_from_json(_ payloadJSON: UnsafePointer<CChar>?) -> Int32 {
    let payload = decodedPayload(from: payloadJSON)
    return StatusBarMenuPresentation(payload: payload).showsAccountChips ? 1 : 0
}

@_cdecl("aah_status_bar_bridge_debug_visible_detail_section_count_from_json")
func aah_status_bar_bridge_debug_visible_detail_section_count_from_json(_ payloadJSON: UnsafePointer<CChar>?) -> Int32 {
    let payload = decodedPayload(from: payloadJSON)
    return StatusBarMenuPresentation(payload: payload).visibleDetailSectionCount
}

@_cdecl("aah_status_bar_bridge_debug_pinned_detail_section_count_from_json")
func aah_status_bar_bridge_debug_pinned_detail_section_count_from_json(_ payloadJSON: UnsafePointer<CChar>?) -> Int32 {
    let payload = decodedPayload(from: payloadJSON)
    return StatusBarMenuPresentation(payload: payload).pinnedDetailSectionCount
}

@_cdecl("aah_status_bar_bridge_debug_scrollable_detail_section_count_from_json")
func aah_status_bar_bridge_debug_scrollable_detail_section_count_from_json(
    _ payloadJSON: UnsafePointer<CChar>?
) -> Int32 {
    let payload = decodedPayload(from: payloadJSON)
    return StatusBarMenuPresentation(payload: payload).scrollableDetailSectionCount
}

@_cdecl("aah_status_bar_bridge_debug_account_chip_layout_axis_from_json")
func aah_status_bar_bridge_debug_account_chip_layout_axis_from_json(_ payloadJSON: UnsafePointer<CChar>?) -> Int32 {
    let payload = decodedPayload(from: payloadJSON)
    return StatusBarMenuPresentation(payload: payload).accountChipLayoutAxisDebugValue
}

@_cdecl("aah_status_bar_bridge_debug_hosting_view_allows_vibrancy")
func aah_status_bar_bridge_debug_hosting_view_allows_vibrancy() -> Int32 {
    StatusBarBridgeController.debugHostingViewAllowsVibrancy() ? 1 : 0
}

@_cdecl("aah_status_bar_bridge_debug_hosting_view_accepts_first_mouse")
func aah_status_bar_bridge_debug_hosting_view_accepts_first_mouse() -> Int32 {
    StatusBarBridgeController.debugHostingViewAcceptsFirstMouse() ? 1 : 0
}

@_cdecl("aah_status_bar_bridge_debug_uses_outer_panel_chrome")
func aah_status_bar_bridge_debug_uses_outer_panel_chrome() -> Int32 {
    StatusBarMenuRootView.usesOuterPanelChrome ? 1 : 0
}

@_cdecl("aah_status_bar_bridge_debug_uses_scrollable_detail_container")
func aah_status_bar_bridge_debug_uses_scrollable_detail_container() -> Int32 {
    StatusBarMenuRootView.usesScrollableDetailContainer ? 1 : 0
}

@_cdecl("aah_status_bar_bridge_debug_menu_refreshes_layout_on_open")
func aah_status_bar_bridge_debug_menu_refreshes_layout_on_open() -> Int32 {
    StatusBarBridgeController.debugRefreshesMenuLayoutOnOpen() ? 1 : 0
}

@_cdecl("aah_status_bar_bridge_debug_panel_width")
func aah_status_bar_bridge_debug_panel_width() -> Int32 {
    Int32(StatusBarPanelTokens.panelWidth)
}

@_cdecl("aah_status_bar_bridge_debug_switcher_height")
func aah_status_bar_bridge_debug_switcher_height() -> Int32 {
    Int32(StatusBarPanelTokens.switcherHeight)
}

@_cdecl("aah_status_bar_bridge_debug_hover_corner_radius")
func aah_status_bar_bridge_debug_hover_corner_radius() -> Int32 {
    Int32(StatusBarPanelTokens.hoverCornerRadius)
}

@_cdecl("aah_status_bar_bridge_debug_footer_action_row_height")
func aah_status_bar_bridge_debug_footer_action_row_height() -> Int32 {
    Int32(StatusBarPanelTokens.actionRowHeight)
}

@_cdecl("aah_status_bar_bridge_debug_footer_icon_column_width")
func aah_status_bar_bridge_debug_footer_icon_column_width() -> Int32 {
    Int32(StatusBarPanelTokens.actionIconColumnWidth)
}

@_cdecl("aah_status_bar_bridge_debug_footer_chevron_column_width")
func aah_status_bar_bridge_debug_footer_chevron_column_width() -> Int32 {
    Int32(StatusBarPanelTokens.actionChevronColumnWidth)
}

@_cdecl("aah_status_bar_bridge_debug_footer_action_horizontal_padding")
func aah_status_bar_bridge_debug_footer_action_horizontal_padding() -> Int32 {
    Int32(StatusBarPanelTokens.footerActionHorizontalPadding)
}

@_cdecl("aah_status_bar_bridge_debug_footer_action_vertical_padding")
func aah_status_bar_bridge_debug_footer_action_vertical_padding() -> Int32 {
    Int32(StatusBarPanelTokens.footerActionVerticalPadding)
}

@_cdecl("aah_status_bar_bridge_debug_panel_content_padding")
func aah_status_bar_bridge_debug_panel_content_padding() -> Int32 {
    Int32(StatusBarPanelTokens.panelContentPadding)
}

@_cdecl("aah_status_bar_bridge_debug_interactive_row_horizontal_padding")
func aah_status_bar_bridge_debug_interactive_row_horizontal_padding() -> Int32 {
    Int32(StatusBarPanelTokens.interactiveRowHorizontalPadding)
}

@_cdecl("aah_status_bar_bridge_debug_interactive_row_vertical_padding")
func aah_status_bar_bridge_debug_interactive_row_vertical_padding() -> Int32 {
    Int32(StatusBarPanelTokens.interactiveRowVerticalPadding)
}

@_cdecl("aah_status_bar_bridge_debug_account_section_hover_horizontal_padding")
func aah_status_bar_bridge_debug_account_section_hover_horizontal_padding() -> Int32 {
    Int32(StatusBarPanelTokens.accountSectionHoverHorizontalPadding)
}

@_cdecl("aah_status_bar_bridge_debug_account_section_hover_vertical_padding")
func aah_status_bar_bridge_debug_account_section_hover_vertical_padding() -> Int32 {
    Int32(StatusBarPanelTokens.accountSectionHoverVerticalPadding)
}

@_cdecl("aah_status_bar_bridge_debug_quota_tone_for_percent")
func aah_status_bar_bridge_debug_quota_tone_for_percent(_ percent: UInt8) -> Int32 {
    StatusBarQuotaTone(percent: percent).rawValue
}
