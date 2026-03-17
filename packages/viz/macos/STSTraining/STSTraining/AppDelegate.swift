import Cocoa
import WebKit

class AppDelegate: NSObject, NSApplicationDelegate {
    var window: NSWindow!
    var viewController: DashboardViewController!

    func applicationDidFinishLaunching(_ notification: Notification) {
        NSApp.appearance = NSAppearance(named: .darkAqua)

        viewController = DashboardViewController()

        window = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 1400, height: 900),
            styleMask: [.titled, .closable, .miniaturizable, .resizable],
            backing: .buffered,
            defer: false
        )
        window.title = "STS Training Dashboard"
        window.minSize = NSSize(width: 1200, height: 800)
        window.contentViewController = viewController
        window.center()
        window.makeKeyAndOrderFront(nil)

        NSApp.activate(ignoringOtherApps: true)
    }

    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
        return true
    }

    func applicationSupportsSecureRestorableState(_ app: NSApplication) -> Bool {
        return true
    }
}

// MARK: - Dashboard View Controller

class DashboardViewController: NSViewController, WKNavigationDelegate {
    private var webView: WKWebView!
    private var retryLabel: NSTextField?
    private var retryTimer: Timer?
    private var retryCount = 0
    private let maxRetries = 120
    private let devURL = URL(string: "http://localhost:5174")!

    override func loadView() {
        view = NSView(frame: NSRect(x: 0, y: 0, width: 1400, height: 900))
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        setupWebView()
        loadDevServer()
    }

    private func setupWebView() {
        let config = WKWebViewConfiguration()
        let prefs = WKWebpagePreferences()
        prefs.allowsContentJavaScript = true
        config.defaultWebpagePreferences = prefs
        config.preferences.setValue(true, forKey: "developerExtrasEnabled")
        config.websiteDataStore = .default()

        webView = WKWebView(frame: view.bounds, configuration: config)
        webView.autoresizingMask = [.width, .height]
        webView.navigationDelegate = self
        webView.setValue(false, forKey: "drawsBackground")

        #if DEBUG
        if #available(macOS 13.3, *) {
            webView.isInspectable = true
        }
        #endif

        view.addSubview(webView)
    }

    private func loadDevServer() {
        let request = URLRequest(
            url: devURL,
            cachePolicy: .reloadIgnoringLocalCacheData,
            timeoutInterval: 3
        )
        webView.load(request)
    }

    // MARK: - Retry UI

    private func showRetryMessage() {
        guard retryLabel == nil else { return }

        let label = NSTextField(labelWithString: "")
        label.font = .monospacedSystemFont(ofSize: 14, weight: .regular)
        label.textColor = .secondaryLabelColor
        label.alignment = .center
        label.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(label)

        NSLayoutConstraint.activate([
            label.centerXAnchor.constraint(equalTo: view.centerXAnchor),
            label.centerYAnchor.constraint(equalTo: view.centerYAnchor),
        ])

        retryLabel = label
        updateRetryText()
    }

    private func updateRetryText() {
        retryLabel?.stringValue = """
            Waiting for Vite dev server at localhost:5174...

            Retry \(retryCount)/\(maxRetries)

            Run: ./scripts/services.sh start
            """
    }

    private func hideRetryMessage() {
        retryLabel?.removeFromSuperview()
        retryLabel = nil
        retryTimer?.invalidate()
        retryTimer = nil
        retryCount = 0
    }

    private func scheduleRetry() {
        showRetryMessage()
        retryTimer?.invalidate()
        retryTimer = Timer.scheduledTimer(withTimeInterval: 1.0, repeats: false) { [weak self] _ in
            guard let self else { return }
            self.retryCount += 1
            if self.retryCount >= self.maxRetries {
                self.retryLabel?.stringValue = """
                    Could not connect to localhost:5174 after \(self.maxRetries) attempts.

                    Start the dev server and relaunch the app.
                    """
                return
            }
            self.updateRetryText()
            self.loadDevServer()
        }
    }

    // MARK: - WKNavigationDelegate

    func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
        hideRetryMessage()
    }

    func webView(_ webView: WKWebView, didFail navigation: WKNavigation!, withError error: Error) {
        scheduleRetry()
    }

    func webView(_ webView: WKWebView, didFailProvisionalNavigation navigation: WKNavigation!, withError error: Error) {
        scheduleRetry()
    }

    deinit {
        retryTimer?.invalidate()
    }
}
