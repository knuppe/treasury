// On-device translation through Apple's Translation framework, exposed to Rust as two C
// functions (src/translate.rs). The framework only has a Swift API, and the direct session
// (`TranslationSession(installedSource:target:)`) needs macOS 26; older systems and language
// pairs that are not downloaded report "unavailable" so the app falls back to the browser.
// The text never leaves the computer. Building it needs the macOS 26 SDK (Xcode 26, Swift 6.2);
// an older toolchain builds a bridge that always reports "unavailable".

import Foundation
import Translation

/// 0 = unavailable (older macOS, unsupported or not downloaded pair), 1 = installed.
@_cdecl("treasury_translation_status")
public func treasuryTranslationStatus(_ source: UnsafePointer<CChar>, _ target: UnsafePointer<CChar>) -> Int32 {
    #if compiler(>=6.2)
    guard #available(macOS 26.0, *) else { return 0 }
    let from = Locale.Language(identifier: String(cString: source))
    let to = Locale.Language(identifier: String(cString: target))
    let done = DispatchSemaphore(value: 0)
    let result = Box<Int32>(0)
    Task.detached {
        let status = await LanguageAvailability().status(from: from, to: to)
        result.value = status == .installed ? 1 : 0
        done.signal()
    }
    done.wait()
    return result.value
    #else
    return 0
    #endif
}

/// Translates `text`. Returns a `malloc`ed UTF-8 string that Rust frees with
/// `treasury_translation_free`; `ok` tells a translation (1) from an error message (0).
@_cdecl("treasury_translate")
public func treasuryTranslate(
    _ source: UnsafePointer<CChar>, _ target: UnsafePointer<CChar>, _ text: UnsafePointer<CChar>,
    _ ok: UnsafeMutablePointer<Int32>
) -> UnsafeMutablePointer<CChar>? {
    ok.pointee = 0
    #if compiler(>=6.2)
    guard #available(macOS 26.0, *) else { return strdup("unavailable") }
    let from = Locale.Language(identifier: String(cString: source))
    let to = Locale.Language(identifier: String(cString: target))
    let input = String(cString: text)
    let done = DispatchSemaphore(value: 0)
    let result = Box<(Bool, String)>((false, "unavailable"))
    Task.detached {
        let session = TranslationSession(installedSource: from, target: to)
        do {
            result.value = (true, try await translateParagraphs(input, session))
        } catch {
            result.value = (false, error.localizedDescription)
        }
        done.signal()
    }
    done.wait()
    ok.pointee = result.value.0 ? 1 : 0
    return strdup(result.value.1)
    #else
    return strdup("unavailable")
    #endif
}

@_cdecl("treasury_translation_free")
public func treasuryTranslationFree(_ ptr: UnsafeMutablePointer<CChar>?) {
    free(ptr)
}

#if compiler(>=6.2)
/// Line by line in one batch, so headings, bullets and blank lines keep their place.
@available(macOS 26.0, *)
private func translateParagraphs(_ input: String, _ session: TranslationSession) async throws -> String {
    let lines = input.components(separatedBy: "\n")
    var requests: [TranslationSession.Request] = []
    for (i, line) in lines.enumerated() where !line.trimmingCharacters(in: .whitespaces).isEmpty {
        requests.append(.init(sourceText: line, clientIdentifier: String(i)))
    }
    var out = lines
    if requests.isEmpty { return input }
    for response in try await session.translations(from: requests) {
        if let id = response.clientIdentifier, let i = Int(id) {
            out[i] = leadingMarker(lines[i]) + stripMarker(response.targetText, like: lines[i])
        }
    }
    return out.joined(separator: "\n")
}

#endif

/// Markdown list and heading markers ("- ", "* ", "## ") are kept from the original line.
private func leadingMarker(_ line: String) -> String {
    let m = line.prefix { $0 == " " || $0 == "-" || $0 == "*" || $0 == "#" }
    return String(m)
}

private func stripMarker(_ translated: String, like original: String) -> String {
    leadingMarker(original).isEmpty
        ? translated
        : String(translated.drop { $0 == " " || $0 == "-" || $0 == "*" || $0 == "#" })
}

private final class Box<T>: @unchecked Sendable {
    var value: T
    init(_ value: T) { self.value = value }
}
