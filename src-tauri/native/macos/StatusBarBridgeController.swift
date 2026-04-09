import AppKit
import Foundation
import SwiftUI

final class StatusBarMenuHostingView<Content: View>: NSHostingView<Content> {
    override var allowsVibrancy: Bool {
        true
    }

    override var isOpaque: Bool {
        false
    }

    override var acceptsFirstResponder: Bool {
        true
    }

    override func acceptsFirstMouse(for event: NSEvent?) -> Bool {
        true
    }

    override func mouseDown(with event: NSEvent) {
        window?.makeKey()
        window?.makeFirstResponder(self)
        super.mouseDown(with: event)
    }

    required init(rootView: Content) {
        super.init(rootView: rootView)
        self.wantsLayer = true
        self.layer?.backgroundColor = NSColor.clear.cgColor
    }

    @available(*, unavailable)
    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}

final class StatusBarMenuSession: ObservableObject {
    @Published private var payload: StatusBarBridgePayload
    @Published private var optimisticTab: StatusBarBridgeTab?

    init(payload: StatusBarBridgePayload = .empty) {
        self.payload = payload
        self.optimisticTab = nil
    }

    var displayedPayload: StatusBarBridgePayload {
        if let optimisticTab {
            return payload.selectingTab(optimisticTab)
        }

        return payload
    }

    func apply(payload: StatusBarBridgePayload) {
        self.payload = payload
        self.optimisticTab = nil
    }

    func applyOptimistic(_ action: StatusBarBridgeAction) {
        guard case .selectTab(let tab) = action else {
            return
        }

        self.optimisticTab = tab
    }
}

enum StatusBarAppIconProvider {
    enum SourceVariant: Int32 {
        case publicAsset = 1
        case bundled = 2
        case repository = 3
        case runtime = 4
        case workspace = 5
        case fallback = 6
    }

    static func load() -> NSImage? {
        loadWithSource()?.image
    }

    static func sourceVariant() -> SourceVariant? {
        loadWithSource()?.variant
    }

    static func load(
        bundleResourceURL: URL?,
        currentDirectoryURL: URL
    ) -> NSImage? {
        loadWithSource(bundleResourceURL: bundleResourceURL, currentDirectoryURL: currentDirectoryURL)?.image
    }

    static func sourceVariant(
        bundleResourceURL: URL?,
        currentDirectoryURL: URL
    ) -> SourceVariant? {
        loadWithSource(bundleResourceURL: bundleResourceURL, currentDirectoryURL: currentDirectoryURL)?.variant
    }

    private static func loadWithSource(
        bundleResourceURL: URL? = Bundle.main.resourceURL,
        currentDirectoryURL: URL = URL(fileURLWithPath: FileManager.default.currentDirectoryPath)
    ) -> (image: NSImage, variant: SourceVariant)? {
        if let publicIcon = publicIconImage(
            bundleResourceURL: bundleResourceURL,
            currentDirectoryURL: currentDirectoryURL
        ) {
            return (publicIcon, .publicAsset)
        }

        if let bundledIcon = iconImage(at: Bundle.main.path(forResource: "icon", ofType: "icns")) {
            return (bundledIcon, .bundled)
        }

        if let bundledPNG = iconImage(at: Bundle.main.path(forResource: "icon", ofType: "png")) {
            return (bundledPNG, .bundled)
        }

        let repositoryCandidates = [
            currentDirectoryURL.appendingPathComponent("src-tauri/icons/icon.icns").path,
            currentDirectoryURL.appendingPathComponent("src-tauri/icons/icon.png").path,
            currentDirectoryURL.appendingPathComponent("icons/icon.icns").path,
            currentDirectoryURL.appendingPathComponent("icons/icon.png").path,
        ]

        for candidate in repositoryCandidates {
            if let repositoryIcon = iconImage(at: candidate) {
                return (repositoryIcon, .repository)
            }
        }

        if let application = NSApp,
           let applicationIcon = iconImage(from: application.applicationIconImage.copy() as? NSImage) {
            return (applicationIcon, .runtime)
        }

        let workspaceIcon = NSWorkspace.shared.icon(forFile: Bundle.main.bundlePath)
        if let preparedWorkspaceIcon = iconImage(from: workspaceIcon) {
            return (preparedWorkspaceIcon, .workspace)
        }

        if let fallbackIcon = iconImage(from: NSImage(named: NSImage.applicationIconName)) {
            return (fallbackIcon, .fallback)
        }

        return nil
    }

    private static func publicIconImage(
        bundleResourceURL: URL?,
        currentDirectoryURL: URL
    ) -> NSImage? {
        let candidates: [String] = [
            Bundle.main.path(forResource: "icon", ofType: "svg"),
            bundleResourceURL?.appendingPathComponent("icon.svg").path,
            bundleResourceURL?.appendingPathComponent("_up_/public/icon.svg").path,
            bundleResourceURL?.appendingPathComponent("public/icon.svg").path,
            currentDirectoryURL.appendingPathComponent("public/icon.svg").path,
            currentDirectoryURL.appendingPathComponent("../public/icon.svg").path,
            currentDirectoryURL.appendingPathComponent("dist/icon.svg").path,
            currentDirectoryURL.appendingPathComponent("../dist/icon.svg").path,
        ].compactMap { $0 }

        return candidates
            .compactMap { iconImage(at: $0, template: true) }
            .first
    }

    private static func iconImage(at path: String?) -> NSImage? {
        iconImage(at: path, template: false)
    }

    private static func iconImage(at path: String?, template: Bool) -> NSImage? {
        guard let path else {
            return nil
        }

        return iconImage(from: NSImage(contentsOfFile: path), template: template)
    }

    private static func iconImage(from image: NSImage?) -> NSImage? {
        iconImage(from: image, template: false)
    }

    private static func iconImage(from image: NSImage?, template: Bool) -> NSImage? {
        guard let copiedImage = image?.copy() as? NSImage,
              copiedImage.size.width > 0,
              copiedImage.size.height > 0 else {
            return nil
        }

        copiedImage.size = NSSize(width: 18, height: 18)
        copiedImage.isTemplate = template
        return copiedImage
    }
}

final class StatusBarBridgeController: NSObject, NSMenuDelegate {
    static let shared = StatusBarBridgeController()

    private var statusItem: NSStatusItem?
    private var callback: AAHStatusBarBridgeCallback?
    private var menu: NSMenu?
    private var panelItem: NSMenuItem?
    private var hostingView: StatusBarMenuHostingView<StatusBarMenuRootView>?
    private let session = StatusBarMenuSession()

    private override init() {}

    func initialize(callback: @escaping AAHStatusBarBridgeCallback) -> Bool {
        self.callback = callback

        if statusItem == nil {
            let item = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
            guard let button = item.button else {
                return false
            }

            button.image = StatusBarAppIconProvider.load()
            button.image?.size = NSSize(width: 18, height: 18)
            button.imagePosition = .imageOnly
            button.imageScaling = .scaleProportionallyDown
            button.toolTip = "AI Accounts Hub"
            statusItem = item
        }

        ensureMenu()
        refreshLayout()
        return true
    }

    func updatePayload(jsonString: String) {
        guard let decodedPayload = StatusBarBridgePayload.decode(jsonString: jsonString) else {
            return
        }

        session.apply(payload: decodedPayload)
        refreshLayout()
    }

    private func ensureMenu() {
        guard let statusItem, menu == nil else {
            return
        }

        let menu = NSMenu(title: "AI Accounts Hub")
        menu.autoenablesItems = false
        menu.delegate = self

        let rootView = makeRootView()
        let hostingView = StatusBarMenuHostingView(rootView: rootView)
        hostingView.frame = NSRect(x: 0, y: 0, width: StatusBarPanelTokens.panelWidth, height: 10)

        let panelItem = NSMenuItem()
        panelItem.view = hostingView
        panelItem.isEnabled = true

        menu.addItem(panelItem)
        statusItem.menu = menu
        self.menu = menu
        self.panelItem = panelItem
        self.hostingView = hostingView
    }

    private func makeRootView(secondaryDetailMaxHeight: CGFloat? = nil) -> StatusBarMenuRootView {
        StatusBarMenuRootView(session: session, secondaryDetailMaxHeight: secondaryDetailMaxHeight) { [weak self] action in
            self?.perform(action)
        }
    }

    private func refreshLayout() {
        ensureMenu()
        guard let hostingView, let panelItem else {
            return
        }

        let naturalContentHeight = measureContentHeight(using: hostingView, secondaryDetailMaxHeight: nil)
        let availablePanelHeight = availablePanelHeight()
        let naturalPanelHeight =
            aah_status_bar_bridge_panel_height_for_content_height(Double(naturalContentHeight))

        var secondaryDetailMaxHeight: CGFloat?
        if naturalPanelHeight > availablePanelHeight {
            let baselineContentHeight = measureContentHeight(using: hostingView, secondaryDetailMaxHeight: 0)
            let allowedContentHeight =
                max(0, availablePanelHeight - StatusBarBridgeSizing.verticalPadding)
            secondaryDetailMaxHeight = CGFloat(max(0, allowedContentHeight - Double(baselineContentHeight)))
        }

        let finalContentHeight =
            measureContentHeight(using: hostingView, secondaryDetailMaxHeight: secondaryDetailMaxHeight)
        let finalPanelHeight = aah_status_bar_bridge_panel_height_clamped_to_available_height(
            Double(finalContentHeight),
            availablePanelHeight
        )

        hostingView.frame = NSRect(x: 0, y: 0, width: StatusBarPanelTokens.panelWidth, height: finalPanelHeight)
        panelItem.view = hostingView
    }

    private func measureContentHeight(
        using hostingView: StatusBarMenuHostingView<StatusBarMenuRootView>,
        secondaryDetailMaxHeight: CGFloat?
    ) -> CGFloat {
        hostingView.rootView = makeRootView(secondaryDetailMaxHeight: secondaryDetailMaxHeight)
        hostingView.layoutSubtreeIfNeeded()
        return hostingView.fittingSize.height
    }

    private func availablePanelHeight() -> Double {
        guard let button = statusItem?.button,
              let window = button.window,
              let screen = window.screen else {
            return Double(NSScreen.main?.visibleFrame.height ?? 720)
        }

        let buttonFrameInWindow = button.convert(button.bounds, to: nil)
        let buttonFrameOnScreen = window.convertToScreen(buttonFrameInWindow)
        let availableBelowButton =
            buttonFrameOnScreen.minY - screen.visibleFrame.minY - StatusBarSizing.panelInset

        return max(Double(availableBelowButton), 0)
    }

    private func applyLocal(_ action: StatusBarBridgeAction) {
        session.applyOptimistic(action)
        refreshLayout()
    }

    private func perform(_ action: StatusBarBridgeAction) {
        if action.keepsMenuOpen {
            applyLocal(action)
            emit(action)
            return
        }

        dismissMenu()

        if action.isHandledLocally {
            handleLocally(action)
            return
        }

        DispatchQueue.main.async { [weak self] in
            self?.emit(action)
        }
    }

    private func dismissMenu() {
        guard let menu else {
            return
        }

        menu.cancelTracking()
    }

    private func handleLocally(_ action: StatusBarBridgeAction) {
        switch action {
        case .quit:
            DispatchQueue.main.async {
                NSApp.terminate(nil)
            }
        case .selectTab, .switchAccount, .refresh, .openMainWindow:
            break
        }
    }

    static func debugHostingViewAllowsVibrancy() -> Bool {
        StatusBarMenuHostingView(rootView: StatusBarMenuRootView(session: StatusBarMenuSession()) { _ in }).allowsVibrancy
    }

    static func debugHostingViewAcceptsFirstMouse() -> Bool {
        StatusBarMenuHostingView(rootView: StatusBarMenuRootView(session: StatusBarMenuSession()) { _ in })
            .acceptsFirstMouse(for: nil)
    }

    static func debugActionKeepsMenuOpen(_ action: StatusBarBridgeAction) -> Bool {
        action.keepsMenuOpen
    }

    static func debugActionIsHandledLocally(_ action: StatusBarBridgeAction) -> Bool {
        action.isHandledLocally
    }

    static func debugRefreshesMenuLayoutOnOpen() -> Bool {
        StatusBarBridgeController.shared.responds(to: #selector(NSMenuDelegate.menuWillOpen(_:)))
    }

    private func emit(_ action: StatusBarBridgeAction) {
        guard let callback, let jsonString = action.jsonString else {
            return
        }

        jsonString.withCString { callback($0) }
    }

    func menuWillOpen(_ menu: NSMenu) {
        refreshLayout()
    }
}

private enum StatusBarBridgeSizing {
    static let verticalPadding: Double = 0
}
