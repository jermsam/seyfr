import SwiftUI

enum TransferStatus: Equatable {
    case idle
    case sending
    case receiving
    case success(String)
    case error(String)
}

@MainActor
class AppState: ObservableObject {
    @Published var ticket: String = ""
    @Published var sendStatus: TransferStatus = .idle
    @Published var receiveStatus: TransferStatus = .idle
    @Published var selectedFileName: String?
    @Published var destinationURL: URL?

    let core: Core

    init() {
        let appSupport = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first!
        let dataDir = appSupport.appendingPathComponent("seyfr")
        try? FileManager.default.createDirectory(at: dataDir, withIntermediateDirectories: true)

        let coreDataDir: String = dataDir.path
        do {
            self.core = try Core(dataDir: coreDataDir)
        } catch {
            fatalError("Failed to initialize Seyfr core: \(error)")
        }

        self.destinationURL = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first
    }

    func send(url: URL) {
        guard url.startAccessingSecurityScopedResource() else {
            sendStatus = .error("Cannot access file")
            return
        }
        defer { url.stopAccessingSecurityScopedResource() }

        selectedFileName = url.lastPathComponent
        sendStatus = .sending

        Task {
            do {
                let result = try core.send(path: url.path, progress: nil)
                await MainActor.run {
                    ticket = result
                    sendStatus = .success("Ready to share")
                }
            } catch {
                await MainActor.run {
                    sendStatus = .error(error.localizedDescription)
                }
            }
        }
    }

    func receive(ticket: String) {
        guard !ticket.isEmpty else {
            receiveStatus = .error("Ticket is empty")
            return
        }

        let dest = destinationURL?.path ?? (FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first?.appendingPathComponent("received").path ?? "/tmp/received")

        receiveStatus = .receiving
        Task {
            do {
                try core.receive(ticket: ticket, destDir: dest, progress: nil)
                await MainActor.run {
                    receiveStatus = .success("Received successfully")
                }
            } catch {
                await MainActor.run {
                    receiveStatus = .error(error.localizedDescription)
                }
            }
        }
    }

    func setDestination(url: URL) {
        destinationURL = url
    }
    
    func clearSend() {
        ticket = ""
        selectedFileName = nil
        sendStatus = .idle
    }
}
