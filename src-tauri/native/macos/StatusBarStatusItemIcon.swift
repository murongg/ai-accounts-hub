import AppKit
import Foundation

private enum StatusBarStatusItemIconPalette {
    case light
    case dark

    init(appearance: NSAppearance?) {
        if appearance?.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua {
            self = .dark
        } else {
            self = .light
        }
    }

    private var monochromeHex: String {
        switch self {
        case .light:
            return "#111111"
        case .dark:
            return "#FFFFFF"
        }
    }

    var accentHex: String { monochromeHex }
    var outlineHex: String { monochromeHex }
    var sparkHex: String { monochromeHex }

    var isMonochrome: Bool {
        accentHex == outlineHex && outlineHex == sparkHex
    }
}

enum StatusBarStatusItemIconRenderer {
    enum FillPathVariant: Int32 {
        case accent = 0
        case fullSilhouette = 1
    }

    enum FillDirectionVariant: Int32 {
        case horizontal = 0
        case majorAxis = 1
    }

    private static let fillPathVariant: FillPathVariant = .fullSilhouette
    private static let fillDirectionVariant: FillDirectionVariant = .majorAxis
    private static let accentPath =
        "M195.2 534.4l147.2-147.2c73.6-1.6 140.8 28.8 203.2 91.2 62.4 62.4 92.8 129.6 91.2 203.2l-147.2 147.2c-81.6 81.6-212.8 81.6-294.4 0s-81.6-212.8 0-294.4z"
    private static let fullSilhouettePath =
        "M195.2 534.4l339.2-339.2c81.6-81.6 212.8-81.6 294.4 0s81.6 212.8 0 294.4L489.6 828.8c-81.6 81.6-212.8 81.6-294.4 0s-81.6-212.8 0-294.4z"
    private static let outlinePath =
        "M217.6 556.8c-68.8 68.8-68.8 180.8 0 249.6s180.8 68.8 249.6 0l339.2-339.2c68.8-68.8 68.8-180.8 0-249.6s-180.8-68.8-249.6 0L217.6 556.8z m-22.4-22.4l339.2-339.2c81.6-81.6 212.8-81.6 294.4 0s81.6 212.8 0 294.4L489.6 828.8c-81.6 81.6-212.8 81.6-294.4 0s-81.6-212.8 0-294.4z"
    private static let sparkPath =
        "M590.4 433.6c-12.8 12.8-32 12.8-44.8 0-12.8-12.8-12.8-32 0-44.8 12.8-12.8 32-12.8 44.8 0 12.8 11.2 12.8 32 0 44.8z m136 67.2c-12.8 12.8-32 12.8-44.8 0s-12.8-32 0-44.8 32-12.8 44.8 0 12.8 32 0 44.8z m17.6-176c-9.6 9.6-24 9.6-33.6 0-9.6-9.6-9.6-24 0-33.6 9.6-9.6 24-9.6 33.6 0s9.6 24 0 33.6z m62.4-16c-6.4 6.4-16 6.4-22.4 0-6.4-6.4-6.4-16 0-22.4 6.4-6.4 16-6.4 22.4 0 6.4 4.8 6.4 16 0 22.4zM704 251.2c-6.4 6.4-16 6.4-22.4 0s-6.4-16 0-22.4 16-6.4 22.4 0c6.4 6.4 6.4 16 0 22.4z m-22.4 158.4c-6.4 6.4-16 6.4-22.4 0-6.4-6.4-6.4-16 0-22.4 6.4-6.4 16-6.4 22.4 0 6.4 6.4 6.4 16 0 22.4z m-124.8-100.8c-6.4 6.4-16 6.4-22.4 0-6.4-6.4-6.4-16 0-22.4 6.4-6.4 16-6.4 22.4 0 6.4 4.8 6.4 16 0 22.4z m107.2-17.6c-16 16-41.6 16-56 0s-16-41.6 0-56 41.6-16 56 0 16 40 0 56z"
    private static let viewBoxWidth = 1024.0
    private static let viewBoxHeight = 1024.0
    private static let iconSize = NSSize(width: 18, height: 18)

    static func usesDynamicIcon(progress: StatusBarBridgeStatusItemProgress?) -> Bool {
        guard let progress else {
            return false
        }

        return !progress.needsRelogin
    }

    static func debugFillPathVariant() -> FillPathVariant {
        fillPathVariant
    }

    static func debugFillDirectionVariant() -> FillDirectionVariant {
        fillDirectionVariant
    }

    static func debugPaletteIsMonochrome(prefersDark: Bool) -> Bool {
        let palette: StatusBarStatusItemIconPalette = prefersDark ? .dark : .light
        return palette.isMonochrome
    }

    static func accentClipWidth(iconWidth: Double, percent: UInt8) -> Double {
        guard iconWidth > 0 else {
            return 0
        }

        let proportionalWidth = iconWidth * Double(percent) / 100.0
        return min(iconWidth, max(0, proportionalWidth))
    }

    static func image(
        for progress: StatusBarBridgeStatusItemProgress?,
        effectiveAppearance: NSAppearance?
    ) -> NSImage? {
        guard usesDynamicIcon(progress: progress),
              let progress else {
            return nil
        }

        let palette = StatusBarStatusItemIconPalette(appearance: effectiveAppearance)
        return compositedImage(percent: progress.percent, palette: palette)
    }

    static func debugOpaquePixelCount(percent: UInt8) -> Int32 {
        guard let image = compositedImage(percent: percent, palette: .light) else {
            return -1
        }

        return Int32(opaquePixelCount(for: image, rasterSize: NSSize(width: 72, height: 72)))
    }

    private static func compositedImage(
        percent: UInt8,
        palette: StatusBarStatusItemIconPalette
    ) -> NSImage? {
        guard let fillImage = svgImage(from: fillSVGString(palette: palette)),
              let baseImage = svgImage(from: baseSVGString(palette: palette)) else {
            return nil
        }

        let fillWidth = accentClipWidth(iconWidth: Double(iconSize.width), percent: percent)
        let composedImage = NSImage(size: iconSize, flipped: false) { destinationRect in
            NSGraphicsContext.current?.imageInterpolation = .high

            if fillWidth > 0 {
                if fillWidth >= Double(iconSize.width) {
                    fillImage.draw(
                        in: destinationRect,
                        from: NSRect(origin: .zero, size: fillImage.size),
                        operation: .sourceOver,
                        fraction: 1.0
                    )
                } else {
                    NSGraphicsContext.saveGraphicsState()
                    fillClipPath(
                        in: destinationRect,
                        progress: Double(percent) / 100.0
                    ).addClip()
                    fillImage.draw(
                        in: destinationRect,
                        from: NSRect(origin: .zero, size: fillImage.size),
                        operation: .sourceOver,
                        fraction: 1.0
                    )
                    NSGraphicsContext.restoreGraphicsState()
                }
            }

            baseImage.draw(
                in: destinationRect,
                from: NSRect(origin: .zero, size: baseImage.size),
                operation: .sourceOver,
                fraction: 1.0
            )
            return true
        }

        composedImage.isTemplate = false
        return composedImage
    }

    private static func fillClipPath(
        in rect: NSRect,
        progress: Double
    ) -> NSBezierPath {
        let clampedProgress = min(1.0, max(0.0, progress))
        let overscan = max(rect.width, rect.height) * 4.0
        let diagonalInset = min(rect.width, rect.height) * 0.19
        let startPoint = CGPoint(x: rect.minX + diagonalInset, y: rect.minY + diagonalInset)
        let endPoint = CGPoint(x: rect.maxX - diagonalInset, y: rect.maxY - diagonalInset)
        let axis = normalizedVector(from: startPoint, to: endPoint)
        let perpendicular = CGPoint(x: -axis.y, y: axis.x)
        let axisLength = hypot(endPoint.x - startPoint.x, endPoint.y - startPoint.y)
        let boundaryPoint = CGPoint(
            x: startPoint.x + axis.x * axisLength * clampedProgress,
            y: startPoint.y + axis.y * axisLength * clampedProgress
        )
        let behindPoint = CGPoint(
            x: startPoint.x - axis.x * overscan,
            y: startPoint.y - axis.y * overscan
        )

        let path = NSBezierPath()
        path.move(
            to: CGPoint(
                x: behindPoint.x - perpendicular.x * overscan,
                y: behindPoint.y - perpendicular.y * overscan
            )
        )
        path.line(
            to: CGPoint(
                x: behindPoint.x + perpendicular.x * overscan,
                y: behindPoint.y + perpendicular.y * overscan
            )
        )
        path.line(
            to: CGPoint(
                x: boundaryPoint.x + perpendicular.x * overscan,
                y: boundaryPoint.y + perpendicular.y * overscan
            )
        )
        path.line(
            to: CGPoint(
                x: boundaryPoint.x - perpendicular.x * overscan,
                y: boundaryPoint.y - perpendicular.y * overscan
            )
        )
        path.close()
        return path
    }

    private static func svgImage(from svg: String) -> NSImage? {
        guard let data = svg.data(using: .utf8),
              let image = NSImage(data: data),
              image.size.width > 0,
              image.size.height > 0 else {
            return nil
        }

        image.size = iconSize
        image.isTemplate = false
        return image
    }

    private static func fillSVGString(
        palette: StatusBarStatusItemIconPalette
    ) -> String {
        let fillPath: String
        switch fillPathVariant {
        case .accent:
            fillPath = accentPath
        case .fullSilhouette:
            fillPath = fullSilhouettePath
        }

        return """
        <?xml version="1.0" encoding="utf-8"?>
        <svg xmlns="http://www.w3.org/2000/svg" width="800" height="800" viewBox="0 0 1024 1024" fill="none">
          <path fill="\(palette.accentHex)" d="\(fillPath)" />
        </svg>
        """
    }

    private static func baseSVGString(
        palette: StatusBarStatusItemIconPalette
    ) -> String {
        """
        <?xml version="1.0" encoding="utf-8"?>
        <svg xmlns="http://www.w3.org/2000/svg" width="800" height="800" viewBox="0 0 1024 1024" fill="none">
          <path fill="\(palette.outlineHex)" d="\(outlinePath)" />
          <path fill="\(palette.sparkHex)" d="\(sparkPath)" />
        </svg>
        """
    }

    private static func opaquePixelCount(for image: NSImage, rasterSize: NSSize) -> Int {
        let bitmapWidth = Int(rasterSize.width)
        let bitmapHeight = Int(rasterSize.height)
        guard bitmapWidth > 0,
              bitmapHeight > 0,
              let bitmap = NSBitmapImageRep(
                  bitmapDataPlanes: nil,
                  pixelsWide: bitmapWidth,
                  pixelsHigh: bitmapHeight,
                  bitsPerSample: 8,
                  samplesPerPixel: 4,
                  hasAlpha: true,
                  isPlanar: false,
                  colorSpaceName: .deviceRGB,
                  bytesPerRow: 0,
                  bitsPerPixel: 0
              ) else {
            return 0
        }

        bitmap.size = rasterSize
        NSGraphicsContext.saveGraphicsState()
        NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: bitmap)
        NSColor.clear.setFill()
        NSBezierPath(rect: NSRect(origin: .zero, size: rasterSize)).fill()
        image.draw(in: NSRect(origin: .zero, size: rasterSize))
        NSGraphicsContext.restoreGraphicsState()

        var opaquePixels = 0
        for y in 0..<bitmapHeight {
            for x in 0..<bitmapWidth {
                guard let color = bitmap.colorAt(x: x, y: y) else {
                    continue
                }

                if color.alphaComponent > 0.01 {
                    opaquePixels += 1
                }
            }
        }

        return opaquePixels
    }

    private static func normalizedVector(from start: CGPoint, to end: CGPoint) -> CGPoint {
        let deltaX = end.x - start.x
        let deltaY = end.y - start.y
        let length = hypot(deltaX, deltaY)
        guard length > 0 else {
            return CGPoint(x: 1, y: 0)
        }

        return CGPoint(x: deltaX / length, y: deltaY / length)
    }
}
