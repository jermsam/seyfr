import SwiftUI
import UniformTypeIdentifiers
@preconcurrency import AVFoundation

enum TransferTab {
    case send, receive
}

struct AppLogo: View {
    var body: some View {
        VStack(spacing: 10) {
            ZStack {
                Circle()
                    .stroke(.primary, lineWidth: 1.2)
                    .frame(width: 40, height: 40)
                Text("S")
                    .font(.system(size: 18, weight: .medium, design: .rounded))
                    .foregroundStyle(.primary)
            }

            Text("SEYFR")
                .font(.system(size: 28, weight: .thin, design: .rounded))
                .tracking(3)
                .foregroundStyle(.primary)

            Text("Send Your Files Right")
                .font(.system(size: 13, weight: .regular, design: .monospaced))
                .foregroundStyle(.secondary)
                .tracking(1.5)
        }
        .padding(.vertical, 24)
        .frame(maxWidth: .infinity)
    }
}

struct ContentView: View {
    @EnvironmentObject var appState: AppState

    @State private var showingFileImporter = false
    @State private var showingFolderPicker = false
    @State private var showingShareSheet = false
    @State private var isFolderMode = false

    var body: some View {
        NavigationStack {
            VStack(spacing: 0) {
                AppLogo()
                TabView {
                    SendView(
                        showingFileImporter: $showingFileImporter,
                        showingFolderPicker: $showingFolderPicker,
                        showingShareSheet: $showingShareSheet,
                        isFolderMode: $isFolderMode
                    )
                    .tabItem {
                        Label("Send", systemImage: "square.and.arrow.up")
                    }
                    .tag(TransferTab.send)

                    ReceiveView()
                    .tabItem {
                        Label("Receive", systemImage: "square.and.arrow.down")
                    }
                    .tag(TransferTab.receive)
                }
                .tint(.primary)
            }
            .background(Color(.systemBackground))
            .fileImporter(
                isPresented: $showingFileImporter,
                allowedContentTypes: [UTType.data],
                allowsMultipleSelection: false
            ) { result in
                switch result {
                case .success(let urls):
                    if let url = urls.first {
                        appState.send(url: url)
                    }
                case .failure(let error):
                    appState.status = .error(error.localizedDescription)
                }
            }
            .sheet(isPresented: $showingShareSheet) {
                if !appState.ticket.isEmpty {
                    ShareSheet(items: [appState.ticket])
                }
            }
            .sheet(isPresented: $showingFolderPicker) {
                FolderPicker { url in
                    appState.send(url: url)
                    showingFolderPicker = false
                }
            }
        }
    }
}

struct ConcentricRings: View {
    var body: some View {
        ZStack {
            ForEach(0..<8) { i in
                Circle()
                    .stroke(Color(.separator).opacity(0.4), lineWidth: 0.5)
                    .frame(width: 80 + CGFloat(i) * 22, height: 80 + CGFloat(i) * 22)
            }
            Image(systemName: "arrow.up")
                .font(.system(size: 28, weight: .medium))
                .foregroundStyle(.primary)
        }
        .frame(maxWidth: .infinity, minHeight: 280)
    }
}

struct CircularProgress: View {
    let progress: Double
    let lineWidth: CGFloat

    var body: some View {
        ZStack {
            Circle()
                .stroke(Color(.separator).opacity(0.3), lineWidth: lineWidth)
            Circle()
                .trim(from: 0, to: progress)
                .stroke(.primary, style: StrokeStyle(lineWidth: lineWidth, lineCap: .round))
                .rotationEffect(.degrees(-90))
        }
    }
}

struct SendView: View {
    @EnvironmentObject var appState: AppState
    @Binding var showingFileImporter: Bool
    @Binding var showingFolderPicker: Bool
    @Binding var showingShareSheet: Bool
    @Binding var isFolderMode: Bool

    var body: some View {
        ScrollView {
            VStack(spacing: 32) {
                if case .idle = appState.status, appState.selectedFileName == nil {
                    VStack(spacing: 0) {
                        Button {
                            if isFolderMode {
                                showingFolderPicker = true
                            } else {
                                showingFileImporter = true
                            }
                        } label: {
                            ZStack {
                                ForEach(0..<8) { i in
                                    Circle()
                                        .stroke(Color(.separator).opacity(0.4), lineWidth: 0.5)
                                        .frame(width: 80 + CGFloat(i) * 22, height: 80 + CGFloat(i) * 22)
                                }
                                Image(systemName: isFolderMode ? "folder" : "doc")
                                    .font(.system(size: 28, weight: .medium))
                                    .foregroundStyle(.primary)
                            }
                            .frame(maxWidth: .infinity, minHeight: 280)
                        }
                        .buttonStyle(PlainButtonStyle())
                        .padding(.horizontal, 20)
                        
                        HStack(spacing: 12) {
                            Text("File mode")
                                .font(.system(size: 13, weight: isFolderMode ? .regular : .semibold, design: .rounded))
                                .foregroundStyle(isFolderMode ? .secondary : .primary)
                            Toggle("", isOn: $isFolderMode)
                                .labelsHidden()
                                .toggleStyle(SwitchToggleStyle(tint: .primary))
                            Text("Folder mode")
                                .font(.system(size: 13, weight: isFolderMode ? .semibold : .regular, design: .rounded))
                                .foregroundStyle(isFolderMode ? .primary : .secondary)
                        }
                        .padding(.top, 16)
                    }
                }

                if let fileName = appState.selectedFileName {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Active Transfers")
                            .font(.system(size: 17, weight: .semibold, design: .rounded))
                            .foregroundStyle(.primary)
                            .padding(.horizontal, 20)

                        FileCard(fileName: fileName, isLoading: appState.status == .sending)
                            .padding(.horizontal, 20)
                            .transition(.move(edge: .bottom).combined(with: .opacity))
                    }
                }

                if !appState.ticket.isEmpty {
                    VStack(alignment: .leading, spacing: 14) {
                        HStack {
                            Label("Transfer Ticket", systemImage: "ticket.fill")
                                .font(.system(size: 15, weight: .semibold, design: .rounded))
                                .foregroundStyle(.primary)
                            
                            Spacer()
                            
                            Button {
                                withAnimation(.spring()) {
                                    appState.clearSend()
                                }
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
                                        .stroke(Color(.separator), lineWidth: 0.5)
                                )
                            }
                        }

                        // QR Code
                        QRCodeView(ticket: appState.ticket)
                            .frame(maxWidth: .infinity)
                            .padding(.vertical, 8)

                        Text(appState.ticket)
                            .font(.system(.footnote, design: .monospaced))
                            .padding(14)
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .background(
                                RoundedRectangle(cornerRadius: 12, style: .continuous)
                                    .stroke(Color(.separator), lineWidth: 0.5)
                            )
                            .textSelection(.enabled)

                        HStack(spacing: 12) {
                            SecondaryButton(title: "Copy", icon: "doc.on.doc") {
                                UIPasteboard.general.string = appState.ticket
                                withAnimation(.spring()) {
                                    appState.status = .success("Copied to clipboard")
                                }
                            }

                            PrimaryButton(title: "Share", icon: "square.and.arrow.up") {
                                showingShareSheet = true
                            }
                        }
                    }
                    .padding(20)
                    .background(
                        RoundedRectangle(cornerRadius: 20, style: .continuous)
                            .stroke(Color(.separator), lineWidth: 0.5)
                    )
                    .padding(.horizontal, 20)
                    .transition(.move(edge: .bottom).combined(with: .opacity))
                }

                StatusPill(status: appState.status)
                    .padding(.horizontal, 20)
            }
            .padding(.vertical, 20)
        }
        .animation(.spring(response: 0.4, dampingFraction: 0.8), value: appState.status)
        .animation(.spring(response: 0.4, dampingFraction: 0.8), value: appState.ticket)
        .animation(.spring(response: 0.4, dampingFraction: 0.8), value: appState.selectedFileName)
    }
}

struct ReceiveView: View {
    @EnvironmentObject var appState: AppState
    @State private var ticketInput = ""
    @State private var showingFileExporter = false
    @State private var exportURL: URL?
    @State private var showingQRScanner = false
    @State private var showingFolderPicker = false

    var body: some View {
        ScrollView {
            VStack(spacing: 28) {
                VStack(spacing: 16) {
                    ZStack {
                        ForEach(0..<8) { i in
                            Circle()
                                .stroke(Color(.separator).opacity(0.4), lineWidth: 0.5)
                                .frame(width: 80 + CGFloat(i) * 22, height: 80 + CGFloat(i) * 22)
                        }
                        Image(systemName: "qrcode.viewfinder")
                            .font(.system(size: 32, weight: .medium))
                            .foregroundStyle(.primary)
                    }
                    .frame(maxWidth: .infinity, minHeight: 280)
                    .contentShape(Rectangle())
                    .onTapGesture {
                        showingQRScanner = true
                    }

                    VStack(spacing: 6) {
                        Text("Receive a file")
                            .font(.system(size: 17, weight: .semibold, design: .rounded))
                            .foregroundStyle(.primary)
                        Text("Tap to scan a QR code or paste below")
                            .font(.system(size: 13, weight: .regular))
                            .foregroundStyle(.secondary)
                    }
                }
                .padding(.top, 12)

                VStack(alignment: .leading, spacing: 14) {
                    HStack {
                        Label("Ticket", systemImage: "ticket.fill")
                            .font(.system(size: 15, weight: .semibold, design: .rounded))
                            .foregroundStyle(.primary)

                        Spacer()

                        HStack(spacing: 8) {
                            Button {
                                if let pasted = UIPasteboard.general.string {
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
                                        .stroke(Color(.separator), lineWidth: 0.5)
                                )
                            }

                            Button {
                                ticketInput = ""
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
                                        .stroke(Color(.separator), lineWidth: 0.5)
                                )
                            }
                        }
                    }

                    TextField("Paste ticket here...", text: $ticketInput, axis: .vertical)
                        .font(.system(.footnote, design: .monospaced))
                        .lineLimit(3...6)
                        .padding(14)
                        .background(
                            RoundedRectangle(cornerRadius: 12, style: .continuous)
                                .stroke(Color(.separator), lineWidth: 0.5)
                        )
                }
                .padding(20)
                .background(
                    RoundedRectangle(cornerRadius: 20, style: .continuous)
                        .stroke(Color(.separator), lineWidth: 0.5)
                )
                .padding(.horizontal, 20)
                .sheet(isPresented: $showingQRScanner) {
                    QRScannerView { scanned in
                        ticketInput = scanned
                        showingQRScanner = false
                    } onDismiss: {
                        showingQRScanner = false
                    }
                }

                VStack(alignment: .leading, spacing: 14) {
                    Label("Save Location", systemImage: "folder.fill")
                        .font(.system(size: 15, weight: .semibold, design: .rounded))
                        .foregroundStyle(.primary)

                    HStack(spacing: 14) {
                        Image(systemName: "folder")
                            .font(.title3)
                            .foregroundStyle(.primary)

                        VStack(alignment: .leading, spacing: 2) {
                            Text(appState.destinationURL?.lastPathComponent ?? "Downloads")
                                .font(.system(size: 15, weight: .medium, design: .rounded))
                            Text(appState.destinationURL?.path ?? "Documents/Downloads")
                                .font(.system(size: 13, weight: .regular))
                                .foregroundStyle(.secondary)
                                .lineLimit(1)
                        }

                        Spacer()

                        SecondaryButton(title: "Change", icon: "chevron.right") {
                            showingFolderPicker = true
                        }
                    }
                    .padding(16)
                    .background(
                        RoundedRectangle(cornerRadius: 14, style: .continuous)
                            .stroke(Color(.separator), lineWidth: 0.5)
                    )
                }
                .padding(20)
                .background(
                    RoundedRectangle(cornerRadius: 20, style: .continuous)
                        .stroke(Color(.separator), lineWidth: 0.5)
                )
                .padding(.horizontal, 20)
                .sheet(isPresented: $showingFolderPicker) {
                    FolderPicker { url in
                        appState.setDestination(url: url)
                        showingFolderPicker = false
                    }
                }

                PrimaryButton(title: "Receive File", icon: "arrow.down.circle") {
                    appState.receive(ticket: ticketInput)
                }
                .disabled(ticketInput.isEmpty)
                .padding(.horizontal, 20)

                StatusPill(status: appState.status)
                    .padding(.horizontal, 20)
            }
            .padding(.vertical, 20)
        }
        .animation(.spring(response: 0.4, dampingFraction: 0.8), value: appState.status)
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
                    .foregroundStyle(.primary)
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
                .stroke(Color(.separator), lineWidth: 0.5)
        )
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
                        .tint(.primary)
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
                    .stroke(Color(.separator), lineWidth: 0.5)
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
                .padding(.vertical, 14)
                .background(
                    RoundedRectangle(cornerRadius: 16, style: .continuous)
                        .stroke(.primary, lineWidth: 1)
                )
        }
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
                .padding(.vertical, 14)
                .background(
                    RoundedRectangle(cornerRadius: 16, style: .continuous)
                        .stroke(Color(.separator), lineWidth: 0.5)
                )
        }
    }
}

class QRScannerViewController: UIViewController, AVCaptureMetadataOutputObjectsDelegate {
    nonisolated(unsafe) var captureSession: AVCaptureSession?
    nonisolated(unsafe) var previewLayer: AVCaptureVideoPreviewLayer?
    nonisolated(unsafe) var onScanned: ((String) -> Void)?
    nonisolated(unsafe) var onDismiss: (() -> Void)?
    private var scanLineView: UIView?
    private var scanLineAnimation: CABasicAnimation?
    private var torchButton: UIButton?

    override func viewDidLoad() {
        super.viewDidLoad()
        checkPermissionAndSetup()
    }

    private func checkPermissionAndSetup() {
        #if targetEnvironment(simulator)
        setupUI()
        showSimulatorMock()
        return
        #endif

        let status = AVCaptureDevice.authorizationStatus(for: .video)
        switch status {
        case .authorized:
            setupCamera()
            setupUI()
            startScanLineAnimation()
        case .notDetermined:
            AVCaptureDevice.requestAccess(for: .video) { [weak self] granted in
                DispatchQueue.main.async {
                    if granted {
                        self?.setupCamera()
                        self?.setupUI()
                        self?.startScanLineAnimation()
                    } else {
                        self?.setupUI()
                        self?.showPermissionDenied()
                    }
                }
            }
        case .denied, .restricted:
            setupUI()
            showPermissionDenied()
        @unknown default:
            setupUI()
            showPermissionDenied()
        }
    }

    private func showSimulatorMock() {
        let blur = UIVisualEffectView(effect: UIBlurEffect(style: .systemUltraThinMaterialDark))
        blur.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(blur)
        NSLayoutConstraint.activate([
            blur.topAnchor.constraint(equalTo: view.topAnchor),
            blur.bottomAnchor.constraint(equalTo: view.bottomAnchor),
            blur.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            blur.trailingAnchor.constraint(equalTo: view.trailingAnchor)
        ])

        let icon = UIImageView(image: UIImage(systemName: "qrcode"))
        icon.tintColor = .white
        icon.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(icon)

        let title = UILabel()
        title.text = "Simulator Mode"
        title.font = UIFont.systemFont(ofSize: 20, weight: .semibold)
        title.textColor = .white
        title.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(title)

        let message = UILabel()
        message.text = "Camera unavailable on Simulator. Enter a ticket string to simulate a scan."
        message.font = UIFont.systemFont(ofSize: 15, weight: .regular)
        message.textColor = .white.withAlphaComponent(0.7)
        message.numberOfLines = 0
        message.textAlignment = .center
        message.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(message)

        let textField = UITextField()
        textField.placeholder = "Paste ticket string here..."
        textField.font = UIFont.systemFont(ofSize: 15, weight: .regular)
        textField.textColor = .black
        textField.backgroundColor = .white
        textField.layer.cornerRadius = 12
        textField.leftView = UIView(frame: CGRect(x: 0, y: 0, width: 16, height: 48))
        textField.leftViewMode = .always
        textField.rightView = UIView(frame: CGRect(x: 0, y: 0, width: 16, height: 48))
        textField.rightViewMode = .always
        textField.translatesAutoresizingMaskIntoConstraints = false
        textField.tag = 99
        view.addSubview(textField)

        let scanButton = UIButton(type: .system)
        scanButton.setTitle("Simulate Scan", for: .normal)
        scanButton.setTitleColor(.black, for: .normal)
        scanButton.titleLabel?.font = UIFont.systemFont(ofSize: 17, weight: .semibold)
        scanButton.backgroundColor = .white
        scanButton.layer.cornerRadius = 12
        scanButton.addTarget(self, action: #selector(simulateScanTapped), for: .touchUpInside)
        scanButton.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(scanButton)

        NSLayoutConstraint.activate([
            icon.centerXAnchor.constraint(equalTo: view.centerXAnchor),
            icon.centerYAnchor.constraint(equalTo: view.centerYAnchor, constant: -120),
            icon.widthAnchor.constraint(equalToConstant: 56),
            icon.heightAnchor.constraint(equalToConstant: 56),

            title.topAnchor.constraint(equalTo: icon.bottomAnchor, constant: 20),
            title.centerXAnchor.constraint(equalTo: view.centerXAnchor),

            message.topAnchor.constraint(equalTo: title.bottomAnchor, constant: 8),
            message.leadingAnchor.constraint(equalTo: view.leadingAnchor, constant: 40),
            message.trailingAnchor.constraint(equalTo: view.trailingAnchor, constant: -40),

            textField.topAnchor.constraint(equalTo: message.bottomAnchor, constant: 28),
            textField.leadingAnchor.constraint(equalTo: view.leadingAnchor, constant: 40),
            textField.trailingAnchor.constraint(equalTo: view.trailingAnchor, constant: -40),
            textField.heightAnchor.constraint(equalToConstant: 48),

            scanButton.topAnchor.constraint(equalTo: textField.bottomAnchor, constant: 16),
            scanButton.centerXAnchor.constraint(equalTo: view.centerXAnchor),
            scanButton.widthAnchor.constraint(equalToConstant: 200),
            scanButton.heightAnchor.constraint(equalToConstant: 48)
        ])
    }

    @objc private func simulateScanTapped() {
        guard let textField = view.viewWithTag(99) as? UITextField,
              let text = textField.text, !text.isEmpty else { return }
        let feedback = UIImpactFeedbackGenerator(style: .light)
        feedback.impactOccurred()
        onScanned?(text)
    }

    private func setupCamera() {
        let session = AVCaptureSession()
        captureSession = session

        guard let videoDevice = AVCaptureDevice.default(for: .video) else { return }

        do {
            try videoDevice.lockForConfiguration()
            if videoDevice.isFocusModeSupported(.continuousAutoFocus) {
                videoDevice.focusMode = .continuousAutoFocus
            }
            if videoDevice.isExposureModeSupported(.continuousAutoExposure) {
                videoDevice.exposureMode = .continuousAutoExposure
            }
            videoDevice.unlockForConfiguration()
        } catch { }

        guard let videoInput = try? AVCaptureDeviceInput(device: videoDevice),
              session.canAddInput(videoInput) else { return }

        session.addInput(videoInput)

        let metadataOutput = AVCaptureMetadataOutput()
        guard session.canAddOutput(metadataOutput) else { return }
        session.addOutput(metadataOutput)
        metadataOutput.setMetadataObjectsDelegate(self, queue: DispatchQueue.main)
        metadataOutput.metadataObjectTypes = [.qr]

        let preview = AVCaptureVideoPreviewLayer(session: session)
        preview.videoGravity = .resizeAspectFill
        preview.frame = view.layer.bounds
        view.layer.insertSublayer(preview, at: 0)
        previewLayer = preview

        DispatchQueue.global(qos: .userInitiated).async {
            session.startRunning()
        }
    }

    private func setupUI() {
        let closeButton = UIButton(type: .system)
        closeButton.setImage(UIImage(systemName: "xmark"), for: .normal)
        closeButton.tintColor = .white
        closeButton.addTarget(self, action: #selector(dismissTapped), for: .touchUpInside)
        closeButton.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(closeButton)

        NSLayoutConstraint.activate([
            closeButton.topAnchor.constraint(equalTo: view.safeAreaLayoutGuide.topAnchor, constant: 16),
            closeButton.trailingAnchor.constraint(equalTo: view.safeAreaLayoutGuide.trailingAnchor, constant: -20),
            closeButton.widthAnchor.constraint(equalToConstant: 44),
            closeButton.heightAnchor.constraint(equalToConstant: 44)
        ])

        if let videoDevice = AVCaptureDevice.default(for: .video), videoDevice.hasTorch {
            let torchBtn = UIButton(type: .system)
            torchBtn.setImage(UIImage(systemName: "bolt.slash.fill"), for: .normal)
            torchBtn.tintColor = .white
            torchBtn.addTarget(self, action: #selector(toggleTorch), for: .touchUpInside)
            torchBtn.translatesAutoresizingMaskIntoConstraints = false
            view.addSubview(torchBtn)
            torchButton = torchBtn

            NSLayoutConstraint.activate([
                torchBtn.topAnchor.constraint(equalTo: view.safeAreaLayoutGuide.topAnchor, constant: 16),
                torchBtn.leadingAnchor.constraint(equalTo: view.safeAreaLayoutGuide.leadingAnchor, constant: 20),
                torchBtn.widthAnchor.constraint(equalToConstant: 44),
                torchBtn.heightAnchor.constraint(equalToConstant: 44)
            ])
        }

        let label = UILabel()
        label.text = "Scan a ticket QR code"
        label.font = UIFont.systemFont(ofSize: 15, weight: .medium)
        label.textColor = .white
        label.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(label)

        NSLayoutConstraint.activate([
            label.bottomAnchor.constraint(equalTo: view.safeAreaLayoutGuide.bottomAnchor, constant: -50),
            label.centerXAnchor.constraint(equalTo: view.centerXAnchor)
        ])

        let scanArea: CGFloat = 260
        let overlay = ScannerOverlayView(frame: CGRect(x: 0, y: 0, width: scanArea, height: scanArea))
        overlay.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(overlay)

        NSLayoutConstraint.activate([
            overlay.centerXAnchor.constraint(equalTo: view.centerXAnchor),
            overlay.centerYAnchor.constraint(equalTo: view.centerYAnchor, constant: -20),
            overlay.widthAnchor.constraint(equalToConstant: scanArea),
            overlay.heightAnchor.constraint(equalToConstant: scanArea)
        ])

        let line = UIView()
        line.backgroundColor = UIColor.white.withAlphaComponent(0.8)
        line.translatesAutoresizingMaskIntoConstraints = false
        line.layer.cornerRadius = 1
        overlay.addSubview(line)
        scanLineView = line

        NSLayoutConstraint.activate([
            line.leadingAnchor.constraint(equalTo: overlay.leadingAnchor, constant: 8),
            line.trailingAnchor.constraint(equalTo: overlay.trailingAnchor, constant: -8),
            line.heightAnchor.constraint(equalToConstant: 2),
            line.topAnchor.constraint(equalTo: overlay.topAnchor, constant: 10)
        ])
    }

    private func startScanLineAnimation() {
        guard let line = scanLineView, let container = line.superview else { return }
        line.layoutIfNeeded()

        let animation = CABasicAnimation(keyPath: "position.y")
        animation.fromValue = 12
        animation.toValue = container.bounds.height - 12
        animation.duration = 2.5
        animation.autoreverses = true
        animation.repeatCount = .infinity
        animation.timingFunction = CAMediaTimingFunction(name: .easeInEaseOut)
        line.layer.add(animation, forKey: "scanLine")
        scanLineAnimation = animation
    }

    private func showPermissionDenied() {
        let blur = UIVisualEffectView(effect: UIBlurEffect(style: .systemUltraThinMaterialDark))
        blur.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(blur)
        NSLayoutConstraint.activate([
            blur.topAnchor.constraint(equalTo: view.topAnchor),
            blur.bottomAnchor.constraint(equalTo: view.bottomAnchor),
            blur.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            blur.trailingAnchor.constraint(equalTo: view.trailingAnchor)
        ])

        let icon = UIImageView(image: UIImage(systemName: "camera.fill"))
        icon.tintColor = .white
        icon.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(icon)

        let title = UILabel()
        title.text = "Camera Access Required"
        title.font = UIFont.systemFont(ofSize: 20, weight: .semibold)
        title.textColor = .white
        title.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(title)

        let message = UILabel()
        message.text = "Please allow camera access in Settings to scan QR codes."
        message.font = UIFont.systemFont(ofSize: 15, weight: .regular)
        message.textColor = .white.withAlphaComponent(0.7)
        message.numberOfLines = 0
        message.textAlignment = .center
        message.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(message)

        let button = UIButton(type: .system)
        button.setTitle("Open Settings", for: .normal)
        button.setTitleColor(.black, for: .normal)
        button.titleLabel?.font = UIFont.systemFont(ofSize: 17, weight: .semibold)
        button.backgroundColor = .white
        button.layer.cornerRadius = 12
        button.addTarget(self, action: #selector(openSettings), for: .touchUpInside)
        button.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(button)

        NSLayoutConstraint.activate([
            icon.centerXAnchor.constraint(equalTo: view.centerXAnchor),
            icon.centerYAnchor.constraint(equalTo: view.centerYAnchor, constant: -80),
            icon.widthAnchor.constraint(equalToConstant: 56),
            icon.heightAnchor.constraint(equalToConstant: 56),

            title.topAnchor.constraint(equalTo: icon.bottomAnchor, constant: 20),
            title.centerXAnchor.constraint(equalTo: view.centerXAnchor),

            message.topAnchor.constraint(equalTo: title.bottomAnchor, constant: 8),
            message.leadingAnchor.constraint(equalTo: view.leadingAnchor, constant: 40),
            message.trailingAnchor.constraint(equalTo: view.trailingAnchor, constant: -40),

            button.topAnchor.constraint(equalTo: message.bottomAnchor, constant: 28),
            button.centerXAnchor.constraint(equalTo: view.centerXAnchor),
            button.widthAnchor.constraint(equalToConstant: 200),
            button.heightAnchor.constraint(equalToConstant: 48)
        ])
    }

    @objc private func openSettings() {
        if let url = URL(string: UIApplication.openSettingsURLString) {
            UIApplication.shared.open(url)
        }
    }

    @objc private func toggleTorch() {
        guard let device = AVCaptureDevice.default(for: .video), device.hasTorch else { return }
        do {
            try device.lockForConfiguration()
            device.torchMode = device.torchMode == .on ? .off : .on
            device.unlockForConfiguration()
            let iconName = device.torchMode == .on ? "bolt.fill" : "bolt.slash.fill"
            torchButton?.setImage(UIImage(systemName: iconName), for: .normal)
        } catch { }
    }

    @objc func dismissTapped() {
        onDismiss?()
    }

    override func viewDidLayoutSubviews() {
        super.viewDidLayoutSubviews()
        previewLayer?.frame = view.layer.bounds
    }

    nonisolated func metadataOutput(_ output: AVCaptureMetadataOutput, didOutput metadataObjects: [AVMetadataObject], from connection: AVCaptureConnection) {
        guard let object = metadataObjects.first as? AVMetadataMachineReadableCodeObject,
              let stringValue = object.stringValue else { return }
        captureSession?.stopRunning()
        DispatchQueue.main.async {
            let feedback = UIImpactFeedbackGenerator(style: .light)
            feedback.impactOccurred()
            self.onScanned?(stringValue)
        }
    }
}

class ScannerOverlayView: UIView {
    override func draw(_ rect: CGRect) {
        let context = UIGraphicsGetCurrentContext()
        context?.setStrokeColor(UIColor.white.cgColor)
        context?.setLineWidth(2.5)

        let cornerLength: CGFloat = 35
        let w = rect.width
        let h = rect.height

        let corners: [(CGPoint, CGPoint)] = [
            (CGPoint(x: 0, y: cornerLength), CGPoint(x: 0, y: 0)),
            (CGPoint(x: 0, y: 0), CGPoint(x: cornerLength, y: 0)),
            (CGPoint(x: w - cornerLength, y: 0), CGPoint(x: w, y: 0)),
            (CGPoint(x: w, y: 0), CGPoint(x: w, y: cornerLength)),
            (CGPoint(x: w, y: h - cornerLength), CGPoint(x: w, y: h)),
            (CGPoint(x: w, y: h), CGPoint(x: w - cornerLength, y: h)),
            (CGPoint(x: cornerLength, y: h), CGPoint(x: 0, y: h)),
            (CGPoint(x: 0, y: h), CGPoint(x: 0, y: h - cornerLength))
        ]

        for (start, end) in corners {
            context?.move(to: start)
            context?.addLine(to: end)
        }
        context?.strokePath()

        context?.setStrokeColor(UIColor.white.withAlphaComponent(0.15).cgColor)
        context?.setLineWidth(0.5)
        let padding: CGFloat = 16
        let scanRect = CGRect(x: padding, y: padding, width: w - padding * 2, height: h - padding * 2)
        context?.stroke(scanRect)
    }
}

struct QRScannerView: UIViewControllerRepresentable {
    let onScanned: (String) -> Void
    let onDismiss: () -> Void

    func makeUIViewController(context: Context) -> QRScannerViewController {
        let controller = QRScannerViewController()
        controller.onScanned = onScanned
        controller.onDismiss = onDismiss
        return controller
    }

    func updateUIViewController(_ uiViewController: QRScannerViewController, context: Context) {}
}

struct FolderPicker: UIViewControllerRepresentable {
    let onPicked: (URL) -> Void

    func makeCoordinator() -> Coordinator {
        Coordinator(self)
    }

    func makeUIViewController(context: Context) -> UIDocumentPickerViewController {
        let picker = UIDocumentPickerViewController(forOpeningContentTypes: [.folder])
        picker.delegate = context.coordinator
        picker.allowsMultipleSelection = false
        picker.shouldShowFileExtensions = true
        picker.directoryURL = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first
        return picker
    }

    func updateUIViewController(_ uiViewController: UIDocumentPickerViewController, context: Context) {}

    class Coordinator: NSObject, UIDocumentPickerDelegate {
        let parent: FolderPicker

        init(_ parent: FolderPicker) {
            self.parent = parent
        }

        func documentPicker(_ controller: UIDocumentPickerViewController, didPickDocumentsAt urls: [URL]) {
            guard let url = urls.first else { return }
            parent.onPicked(url)
        }
    }
}

struct ShareSheet: UIViewControllerRepresentable {
    let items: [Any]

    func makeUIViewController(context: Context) -> UIActivityViewController {
        UIActivityViewController(activityItems: items, applicationActivities: nil)
    }

    func updateUIViewController(_ uiViewController: UIActivityViewController, context: Context) {}
}

struct QRCodeView: View {
    let ticket: String
    
    var body: some View {
        if let qrImage = generateQRCode(from: ticket) {
            Image(uiImage: qrImage)
                .interpolation(.none)
                .resizable()
                .scaledToFit()
                .frame(width: 200, height: 200)
                .padding(16)
                .background(Color.white)
                .clipShape(RoundedRectangle(cornerRadius: 16, style: .continuous))
        } else {
            RoundedRectangle(cornerRadius: 16, style: .continuous)
                .fill(Color(.systemGray6))
                .frame(width: 200, height: 200)
                .overlay(
                    Image(systemName: "qrcode")
                        .font(.system(size: 48))
                        .foregroundStyle(.secondary)
                )
        }
    }
    
    private func generateQRCode(from string: String) -> UIImage? {
        let data = Data(string.utf8)
        
        guard let filter = CIFilter(name: "CIQRCodeGenerator") else { return nil }
        filter.setValue(data, forKey: "inputMessage")
        filter.setValue("H", forKey: "inputCorrectionLevel") // High error correction
        
        guard let ciImage = filter.outputImage else { return nil }
        
        // Scale up the QR code (it's generated at low resolution)
        let transform = CGAffineTransform(scaleX: 10, y: 10)
        let scaledImage = ciImage.transformed(by: transform)
        
        let context = CIContext()
        guard let cgImage = context.createCGImage(scaledImage, from: scaledImage.extent) else { return nil }
        
        return UIImage(cgImage: cgImage)
    }
}

#Preview {
    ContentView()
        .environmentObject(AppState())
}
