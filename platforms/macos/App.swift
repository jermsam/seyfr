import SwiftUI

class AppDelegate: NSObject, NSApplicationDelegate {
    func applicationDidFinishLaunching(_ notification: Notification) {
        // SwiftUI window may not exist yet; retry until found
        var attempts = 0
        func resize() {
            guard attempts < 20 else { return }
            attempts += 1
            if let window = NSApp.windows.first {
                window.setFrameAutosaveName("")
                let screenFrame = window.screen?.visibleFrame ?? NSRect(x: 0, y: 0, width: 900, height: 700)
                let w: CGFloat = 1000
                let h: CGFloat = 900
                let x = screenFrame.midX - w / 2
                let y = screenFrame.midY - h / 2
                window.setFrame(NSRect(x: x, y: y, width: w, height: h), display: true)
            } else {
                DispatchQueue.main.asyncAfter(deadline: .now() + 0.1, execute: resize)
            }
        }
        resize()
    }
}

@main
struct SeyfrApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) var appDelegate
    @StateObject private var appState = AppState()

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(appState)
                .onAppear {
                    for window in NSApp.windows {
                        window.setFrameAutosaveName("")
                        let screenFrame = window.screen?.visibleFrame ?? NSRect(x: 0, y: 0, width: 1000, height: 900)
                        let w: CGFloat = 1000
                        let h: CGFloat = 900
                        let x = screenFrame.midX - w / 2
                        let y = screenFrame.midY - h / 2
                        window.setFrame(NSRect(x: x, y: y, width: w, height: h), display: true)
                    }
                }
        }
        .windowResizability(.contentSize)
    }
}
