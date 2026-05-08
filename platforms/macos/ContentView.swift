import SwiftUI

enum MacTab {
    case send, receive
}

struct ContentView: View {
    @EnvironmentObject var appState: AppState
    @State private var selectedTab: MacTab = .send
    @State private var isFolderMode = false
    @State private var ticketInput = ""
    @State private var isTargeted = false

    var body: some View {
        NavigationSplitView {
            SidebarView(selectedTab: $selectedTab, appState: appState)
                .frame(minWidth: 200, idealWidth: 220)
        } detail: {
            Group {
                switch selectedTab {
                case .send:
                    SendDetailView(appState: appState, isFolderMode: $isFolderMode, isTargeted: $isTargeted)
                case .receive:
                    ReceiveDetailView(appState: appState, ticketInput: $ticketInput)
                }
            }
        }
        .navigationSplitViewStyle(.balanced)
    }
}

struct SidebarView: View {
    @Binding var selectedTab: MacTab
    @ObservedObject var appState: AppState

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            // Logo
            VStack(alignment: .leading, spacing: 12) {
                ZStack {
                    Circle()
                        .stroke(Color.primary, lineWidth: 1.2)
                        .frame(width: 44, height: 44)
                    Text("S")
                        .font(.system(size: 20, weight: .medium, design: .rounded))
                }

                VStack(alignment: .leading, spacing: 4) {
                    Text("SEYFR")
                        .font(.system(size: 22, weight: .thin, design: .rounded))
                        .tracking(3)
                    Text("Send Your Files Right")
                        .font(.system(size: 11, weight: .regular, design: .monospaced))
                        .foregroundStyle(.secondary)
                        .tracking(1)
                }
            }
            .padding(.horizontal, 16)
            .padding(.top, 20)
            .padding(.bottom, 32)

            // Navigation
            VStack(spacing: 4) {
                NavButton(
                    title: "Send",
                    icon: "arrow.up.circle.fill",
                    isSelected: selectedTab == .send
                ) {
                    selectedTab = .send
                }

                NavButton(
                    title: "Receive",
                    icon: "arrow.down.circle.fill",
                    isSelected: selectedTab == .receive
                ) {
                    selectedTab = .receive
                }
            }
            .padding(.horizontal, 12)

            Spacer()

            // Status
            VStack(alignment: .leading, spacing: 6) {
                HStack(spacing: 6) {
                    Circle()
                        .fill(Color.green)
                        .frame(width: 8, height: 8)
                    Text("Online")
                        .font(.system(size: 12, weight: .semibold))
                }
                Text(selectedTab == .send ? "Ready to send files" : "Ready to receive files")
                    .font(.system(size: 11))
                    .foregroundStyle(.secondary)
            }
            .padding(.horizontal, 16)
            .padding(.bottom, 20)
        }
        .background(Color(.windowBackgroundColor))
    }
}

struct NavButton: View {
    let title: String
    let icon: String
    let isSelected: Bool
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: 10) {
                Image(systemName: icon)
                    .font(.system(size: 16, weight: .medium))
                    .foregroundStyle(isSelected ? Color.accentColor : .secondary)
                Text(title)
                    .font(.system(size: 14, weight: isSelected ? .semibold : .medium))
                    .foregroundStyle(isSelected ? Color.accentColor : .primary)
                Spacer()
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 8)
            .background(isSelected ? Color.accentColor.opacity(0.1) : Color.clear)
            .clipShape(RoundedRectangle(cornerRadius: 8, style: .continuous))
        }
        .buttonStyle(.plain)
    }
}

struct ConcentricRings: View {
    let icon: String
    let iconSize: CGFloat

    var body: some View {
        ZStack {
            ForEach(0..<8) { i in
                Circle()
                    .stroke(Color.primary.opacity(0.25).opacity(0.4), lineWidth: 0.5)
                    .frame(width: 80 + CGFloat(i) * 22, height: 80 + CGFloat(i) * 22)
            }
            Image(systemName: icon)
                .font(.system(size: iconSize, weight: .medium))
                .foregroundStyle(.primary)
        }
        .frame(maxWidth: .infinity, minHeight: 280)
        .contentShape(Rectangle())
    }
}

struct SendDetailView: View {
    @ObservedObject var appState: AppState
    @Binding var isFolderMode: Bool
    @Binding var isTargeted: Bool

    var body: some View {
        ScrollView {
            VStack(spacing: 32) {
                // Header
                VStack(spacing: 8) {
                    Text("Send")
                        .font(.system(size: 28, weight: .semibold, design: .rounded))
                    Text("Send your files to any device")
                        .font(.system(size: 13, weight: .regular))
                        .foregroundStyle(.secondary)
                }

                if case .idle = appState.sendStatus, appState.selectedFileName == nil {
                    // Drop area
                    VStack(spacing: 16) {
                        ZStack {
                            ConcentricRings(icon: isFolderMode ? "folder" : "doc", iconSize: 28)
                                .overlay(
                                    RoundedRectangle(cornerRadius: 20)
                                        .stroke(isTargeted ? Color.accentColor : Color.clear, lineWidth: 2)
                                )
                            Color.clear
                                .contentShape(Rectangle())
                                .onTapGesture {
                                    showPicker(isFolder: isFolderMode)
                                }
                        }

                        VStack(spacing: 4) {
                            Text("Drag & drop files here")
                                .font(.system(size: 14, weight: .regular))
                                .foregroundStyle(.secondary)
                            Button("or click to browse") {
                                showPicker(isFolder: isFolderMode)
                            }
                            .font(.system(size: 14, weight: .medium))
                            .buttonStyle(.plain)
                        }

                        HStack(spacing: 12) {
                            Text("File mode")
                                .font(.system(size: 13, weight: isFolderMode ? .regular : .semibold, design: .rounded))
                                .foregroundStyle(isFolderMode ? .secondary : .primary)
                            Toggle("", isOn: $isFolderMode)
                                .labelsHidden()
                                .toggleStyle(.switch)
                            Text("Folder mode")
                                .font(.system(size: 13, weight: isFolderMode ? .semibold : .regular, design: .rounded))
                                .foregroundStyle(isFolderMode ? .primary : .secondary)
                        }
                        .padding(.top, 8)
                    }
                    .padding(.horizontal, 40)
                    .onDrop(of: [.fileURL], isTargeted: $isTargeted) { providers in
                        guard let provider = providers.first else { return false }
                        _ = provider.loadObject(ofClass: URL.self) { url, _ in
                            if let url = url {
                                DispatchQueue.main.async {
                                    appState.send(url: url)
                                }
                            }
                        }
                        return true
                    }
                }

                if let fileName = appState.selectedFileName {
                    FileCard(fileName: fileName, isLoading: appState.sendStatus == .sending)
                        .padding(.horizontal, 40)
                        .transition(.move(edge: .bottom).combined(with: .opacity))
                }

                if !appState.ticket.isEmpty {
                    TicketCard(
                        ticket: appState.ticket,
                        onCopy: {
                            NSPasteboard.general.clearContents()
                            NSPasteboard.general.setString(appState.ticket, forType: .string)
                            appState.sendStatus = .success("Copied to clipboard")
                        },
                        onClear: { appState.clearSend() }
                    )
                    .padding(.horizontal, 40)
                    .transition(.move(edge: .bottom).combined(with: .opacity))
                }

                StatusPill(status: appState.sendStatus)
                    .padding(.horizontal, 40)
            }
            .padding(.vertical, 32)
        }
        .animation(.spring(response: 0.4, dampingFraction: 0.8), value: appState.sendStatus)
        .animation(.spring(response: 0.4, dampingFraction: 0.8), value: appState.ticket)
    }

    private func showPicker(isFolder: Bool) {
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) {
            NSApp.activate(ignoringOtherApps: true)
            let panel = NSOpenPanel()
            panel.canChooseFiles = !isFolder
            panel.canChooseDirectories = isFolder
            panel.allowsMultipleSelection = false
            panel.canCreateDirectories = false
            panel.message = isFolder ? "Select a folder to send" : "Select a file to send"
            if panel.runModal() == .OK, let url = panel.url {
                appState.send(url: url)
            }
        }
    }
}

struct ReceiveDetailView: View {
    @ObservedObject var appState: AppState
    @Binding var ticketInput: String

    var body: some View {
        ScrollView {
            VStack(spacing: 32) {
                // Header
                VStack(spacing: 8) {
                    Text("Receive")
                        .font(.system(size: 28, weight: .semibold, design: .rounded))
                    Text("Receive files from any device")
                        .font(.system(size: 13, weight: .regular))
                        .foregroundStyle(.secondary)
                }

                // Ticket Card
                VStack(alignment: .leading, spacing: 14) {
                    HStack {
                        Text("Enter ticket")
                            .font(.system(size: 15, weight: .semibold, design: .rounded))
                        Spacer()
                        HStack(spacing: 8) {
                            Button {
                                if let pasted = NSPasteboard.general.string(forType: .string) {
                                    ticketInput = pasted
                                }
                            } label: {
                                HStack(spacing: 4) {
                                    Image(systemName: "doc.on.clipboard")
                                        .font(.system(size: 12, weight: .medium))
                                    Text("Paste")
                                        .font(.system(size: 12, weight: .medium))
                                }
                                .foregroundStyle(.primary)
                                .padding(.horizontal, 10)
                                .padding(.vertical, 6)
                                .background(
                                    RoundedRectangle(cornerRadius: 8, style: .continuous)
                                        .stroke(Color.primary.opacity(0.25), lineWidth: 0.5)
                                )
                            }
                            .buttonStyle(.plain)

                            if !ticketInput.isEmpty {
                                Button {
                                    ticketInput = ""
                                } label: {
                                    HStack(spacing: 4) {
                                        Image(systemName: "xmark.circle.fill")
                                            .font(.system(size: 12, weight: .medium))
                                        Text("Clear")
                                            .font(.system(size: 12, weight: .medium))
                                    }
                                    .foregroundStyle(.primary)
                                    .padding(.horizontal, 10)
                                    .padding(.vertical, 6)
                                    .background(
                                        RoundedRectangle(cornerRadius: 8, style: .continuous)
                                            .stroke(Color.primary.opacity(0.25), lineWidth: 0.5)
                                    )
                                }
                                .buttonStyle(.plain)
                                .transition(.opacity)
                            }
                        }
                    }

                    TextField("Paste ticket here...", text: $ticketInput, axis: .vertical)
                        .font(.system(size: 11, weight: .regular, design: .monospaced))
                        .lineLimit(3...6)
                        .padding(10)
                        .background(
                            RoundedRectangle(cornerRadius: 12, style: .continuous)
                                .stroke(Color.primary.opacity(0.25), lineWidth: 0.5)
                        )
                }
                .padding(20)
                .background(
                    RoundedRectangle(cornerRadius: 20, style: .continuous)
                        .stroke(Color.primary.opacity(0.25), lineWidth: 0.5)
                )
                .padding(.horizontal, 40)

                // Save Location
                VStack(alignment: .leading, spacing: 14) {
                    HStack {
                        Label("Save Location", systemImage: "folder.fill")
                            .font(.system(size: 15, weight: .semibold, design: .rounded))
                        Spacer()
                        Button("Change") {
                            showFolderPicker()
                        }
                        .font(.system(size: 12, weight: .medium))
                        .buttonStyle(.plain)
                    }

                    HStack(spacing: 12) {
                        Image(systemName: "folder")
                            .font(.title3)
                            .foregroundStyle(.secondary)
                        VStack(alignment: .leading, spacing: 2) {
                            Text(appState.destinationURL?.lastPathComponent ?? "Downloads")
                                .font(.system(size: 14, weight: .medium))
                            Text(appState.destinationURL?.path ?? "Documents/Downloads")
                                .font(.system(size: 12))
                                .foregroundStyle(.secondary)
                                .lineLimit(1)
                        }
                        Spacer()
                    }
                    .padding(14)
                    .background(
                        RoundedRectangle(cornerRadius: 14, style: .continuous)
                            .stroke(Color.primary.opacity(0.25), lineWidth: 0.5)
                    )
                }
                .padding(20)
                .background(
                    RoundedRectangle(cornerRadius: 20, style: .continuous)
                        .stroke(Color.primary.opacity(0.25), lineWidth: 0.5)
                )
                .padding(.horizontal, 40)

                PrimaryButton(title: "Receive File", icon: "arrow.down.circle") {
                    appState.receive(ticket: ticketInput)
                }
                .disabled(ticketInput.isEmpty)
                .padding(.horizontal, 40)

                StatusPill(status: appState.receiveStatus)
                    .padding(.horizontal, 40)

                Text("Once you enter a valid ticket, the files will be ready to download.")
                    .font(.system(size: 12))
                    .foregroundStyle(.secondary)
                    .multilineTextAlignment(.center)
                    .padding(.horizontal, 40)
            }
            .padding(.vertical, 32)
        }
        .animation(.spring(response: 0.4, dampingFraction: 0.8), value: appState.receiveStatus)
    }

    private func showFolderPicker() {
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) {
            NSApp.activate(ignoringOtherApps: true)
            let panel = NSOpenPanel()
            panel.canChooseFiles = false
            panel.canChooseDirectories = true
            panel.allowsMultipleSelection = false
            panel.canCreateDirectories = true
            panel.message = "Select a folder to save received files"
            if panel.runModal() == .OK, let url = panel.url {
                appState.setDestination(url: url)
            }
        }
    }
}

struct FileCard: View {
    let fileName: String
    let isLoading: Bool

    var body: some View {
        HStack(spacing: 16) {
            ZStack {
                CircularProgress(progress: isLoading ? 0.5 : 1.0, lineWidth: 2)
                    .frame(width: 40, height: 40)
                if !isLoading {
                    Image(systemName: "checkmark")
                        .font(.system(size: 12, weight: .semibold))
                        .foregroundStyle(.primary)
                }
            }

            VStack(alignment: .leading, spacing: 4) {
                Text(fileName)
                    .font(.system(size: 15, weight: .medium, design: .rounded))
                    .lineLimit(1)
                Text(isLoading ? "In Progress" : "Completed")
                    .font(.system(size: 13, weight: .regular))
                    .foregroundStyle(.secondary)
            }

            Spacer()
        }
        .padding(.vertical, 14)
        .padding(.horizontal, 16)
        .background(
            RoundedRectangle(cornerRadius: 16, style: .continuous)
                .stroke(Color.primary.opacity(0.25), lineWidth: 0.5)
        )
    }
}

struct TicketCard: View {
    let ticket: String
    let onCopy: () -> Void
    let onClear: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            HStack {
                Label("Transfer Ticket", systemImage: "ticket.fill")
                    .font(.system(size: 15, weight: .semibold, design: .rounded))
                Spacer()
                Button {
                    withAnimation(.spring()) { onClear() }
                } label: {
                    HStack(spacing: 4) {
                        Image(systemName: "xmark.circle.fill")
                            .font(.system(size: 12, weight: .medium))
                        Text("Clear")
                            .font(.system(size: 12, weight: .medium))
                    }
                    .foregroundStyle(.secondary)
                    .padding(.horizontal, 10)
                    .padding(.vertical, 6)
                    .background(
                        RoundedRectangle(cornerRadius: 8, style: .continuous)
                            .stroke(Color.primary.opacity(0.25), lineWidth: 0.5)
                    )
                }
                .buttonStyle(.plain)
            }

            // QR Code placeholder — integrate with real QR if available
            QRCodeView(ticket: ticket)
                .frame(maxWidth: .infinity)
                .padding(.vertical, 8)

            Text(ticket)
                .font(.system(size: 11, weight: .regular, design: .monospaced))
                .padding(6)
                .frame(maxWidth: .infinity, alignment: .leading)
                .background(
                    RoundedRectangle(cornerRadius: 12, style: .continuous)
                        .stroke(Color.primary.opacity(0.25), lineWidth: 0.5)
                )
                .textSelection(.enabled)

            HStack(spacing: 12) {
                SecondaryButton(title: "Copy", icon: "doc.on.doc", action: onCopy)
                PrimaryButton(title: "Share", icon: "square.and.arrow.up") {
                    NSPasteboard.general.setString(ticket, forType: .string)
                }
            }
        }
        .padding(20)
        .background(
            RoundedRectangle(cornerRadius: 20, style: .continuous)
                .stroke(Color.primary.opacity(0.25), lineWidth: 0.5)
        )
    }
}

struct CircularProgress: View {
    let progress: Double
    let lineWidth: CGFloat

    var body: some View {
        ZStack {
            Circle()
                .stroke(Color.primary.opacity(0.25).opacity(0.3), lineWidth: lineWidth)
            Circle()
                .trim(from: 0, to: progress)
                .stroke(.primary, style: StrokeStyle(lineWidth: lineWidth, lineCap: .round))
                .rotationEffect(.degrees(-90))
        }
    }
}

struct StatusPill: View {
    let status: TransferStatus

    var body: some View {
        if case .idle = status {
            EmptyView()
        } else {
            HStack(spacing: 10) {
                switch status {
                case .idle:
                    EmptyView()
                case .sending, .receiving:
                    ProgressView()
                        .controlSize(.small)
                    Text(statusText)
                        .foregroundStyle(.primary)
                case .success(let msg):
                    Image(systemName: "checkmark")
                        .font(.system(size: 12, weight: .semibold))
                        .foregroundStyle(.primary)
                    Text(msg)
                        .foregroundStyle(.primary)
                case .error(let msg):
                    Image(systemName: "exclamationmark")
                        .font(.system(size: 12, weight: .semibold))
                        .foregroundStyle(.red)
                    Text(msg)
                        .foregroundStyle(.red)
                }
            }
            .font(.system(size: 13, weight: .medium, design: .rounded))
            .padding(.horizontal, 16)
            .padding(.vertical, 10)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(
                RoundedRectangle(cornerRadius: 12, style: .continuous)
                    .stroke(Color.primary.opacity(0.25), lineWidth: 0.5)
            )
        }
    }

    var statusText: String {
        switch status {
        case .sending: return "Uploading..."
        case .receiving: return "Downloading..."
        default: return ""
        }
    }
}

struct PrimaryButton: View {
    let title: String
    let icon: String
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            Label(title, systemImage: icon)
                .font(.system(size: 15, weight: .semibold, design: .rounded))
                .foregroundStyle(.primary)
                .frame(maxWidth: .infinity)
                .padding(.vertical, 12)
                .background(
                    RoundedRectangle(cornerRadius: 12, style: .continuous)
                        .stroke(.primary, lineWidth: 1)
                )
        }
        .buttonStyle(.plain)
    }
}

struct SecondaryButton: View {
    let title: String
    let icon: String
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            Label(title, systemImage: icon)
                .font(.system(size: 15, weight: .medium, design: .rounded))
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity)
                .padding(.vertical, 12)
                .background(
                    RoundedRectangle(cornerRadius: 12, style: .continuous)
                        .stroke(Color.primary.opacity(0.25), lineWidth: 0.5)
                )
        }
        .buttonStyle(.plain)
    }
}

struct QRCodeView: View {
    let ticket: String

    var body: some View {
        if let qrImage = generateQRCode(from: ticket) {
            Image(nsImage: qrImage)
                .interpolation(.none)
                .resizable()
                .scaledToFit()
                .frame(width: 200, height: 200)
                .padding(16)
                .background(Color.white)
                .clipShape(RoundedRectangle(cornerRadius: 16, style: .continuous))
        } else {
            RoundedRectangle(cornerRadius: 16, style: .continuous)
                .fill(Color(.separatorColor).opacity(0.2))
                .frame(width: 200, height: 200)
                .overlay(
                    Image(systemName: "qrcode")
                        .font(.system(size: 48))
                        .foregroundStyle(.secondary)
                )
        }
    }

    private func generateQRCode(from string: String) -> NSImage? {
        let data = Data(string.utf8)
        guard let filter = CIFilter(name: "CIQRCodeGenerator") else { return nil }
        filter.setValue(data, forKey: "inputMessage")
        filter.setValue("H", forKey: "inputCorrectionLevel")
        guard let ciImage = filter.outputImage else { return nil }
        let transform = CGAffineTransform(scaleX: 10, y: 10)
        let scaledImage = ciImage.transformed(by: transform)
        let context = CIContext()
        guard let cgImage = context.createCGImage(scaledImage, from: scaledImage.extent) else { return nil }
        return NSImage(cgImage: cgImage, size: NSSize(width: 200, height: 200))
    }
}

#Preview {
    ContentView()
        .environmentObject(AppState(preview: true))
}
