using Microsoft.UI.Xaml.Media.Imaging;
using System;
using System.ComponentModel;
using System.ComponentModel.Design;
using System.IO;
using System.Threading.Tasks;
using System.Windows.Input;
using uniffi.seyfr_core;
using WinRT.Interop;
using Windows.Storage.Pickers;
using Windows.Storage.Streams;
using System.Runtime.InteropServices.WindowsRuntime;
using Microsoft.UI.Xaml.Media;

namespace Seyfr
{
    /// <summary>
    /// ViewModel that bridges WinUI XAML with the Rust Core library.
    /// Implements INotifyPropertyChanged for data binding.
    /// </summary>
    /// 
    public enum TransferTab
    {
        Send,
        Receive
    }
    public class AppViewModel : INotifyPropertyChanged
    {
       
        private TransferTab _selectedTab = TransferTab.Send;
        private bool _isFolderMode = false;
        private string? _selectedFileName;
        private string _ticket = "";
        private string _ticketInput = "";
        private string _status = "";
        private bool _isBusy = false;
        private bool _isError = false;
        private string _destinationPath = "";
        private string _destinationName = "";

        private readonly Core _core;

        private string? _selectedFilePath;

        public event PropertyChangedEventHandler? PropertyChanged;

        public TransferTab SelectedTab
        {
            get => _selectedTab;
            set {
                if (_selectedTab != value)
                {
                    _selectedTab = value;
                    OnPropertyChanged(nameof(SelectedTab));
                }
            }

        }

        public bool IsFolderMode
        {
            get => _isFolderMode;
            set
            {
                if (_isFolderMode != value)
                {
                    _isFolderMode = value;
                    OnPropertyChanged(nameof(IsFolderMode));
                }
            }
        }

        public string? SelectedFileName
        {
            get => _selectedFileName;
            private set
            {
                if (_selectedFileName != value)
                {
                    _selectedFileName = value;
                    OnPropertyChanged(nameof(SelectedFileName));
                    OnPropertyChanged(nameof(HasSelectedFile));
                }
            }
        }

        public string? SelectedFilePath
        {
            get => _selectedFilePath;
            private set
            {
                if (_selectedFilePath != value)
                {
                    _selectedFilePath = value;
                    OnPropertyChanged(nameof(SelectedFilePath));
                    OnPropertyChanged(nameof(HasSelectedFile));
                }
            }
        }

        public bool HasSelectedFile => !string.IsNullOrEmpty(_selectedFileName);

        public string Ticket
        {
            get => _ticket;
            private set
            {
                if (_ticket != value)
                {
                    _ticket = value;
                    OnPropertyChanged(nameof(Ticket));
                    OnPropertyChanged(nameof(HasTicket));
                    _ = UpdateQrCodeAsync(value);
                }
            }
        }

        public bool HasTicket => !string.IsNullOrEmpty(_ticket);

        public ImageSource? TicketQrImage => _ticketQrImage;
        private ImageSource? _ticketQrImage;

        private async Task UpdateQrCodeAsync(string ticket)
        {
            if (string.IsNullOrEmpty(ticket))
            {
                _ticketQrImage = null;
                OnPropertyChanged(nameof(TicketQrImage));
                return;
            }

            try
            {
                var bytes = await Task.Run(() => QrCodeHelper.GeneratePngBytes(ticket));
                var bitmap = new BitmapImage();
                var ms = new InMemoryRandomAccessStream();
                using (var writer = new DataWriter(ms.GetOutputStreamAt(0)))
                {
                    writer.WriteBytes(bytes);
                    await writer.StoreAsync();
                    await writer.FlushAsync();
                }
                ms.Seek(0);
                await bitmap.SetSourceAsync(ms);
                _ticketQrImage = bitmap;
            }
            catch (Exception ex)
            {
                _ticketQrImage = null;
                IsError = true;
                Status = $"QR Error: {ex.Message}";
            }
            OnPropertyChanged(nameof(TicketQrImage));
        }

        public string TicketInput
        {
            get => _ticketInput;
            set
            {
                if (_ticketInput != value)
                {
                    _ticketInput = value;
                    OnPropertyChanged(nameof(TicketInput));
                    OnPropertyChanged(nameof(HasTicketInput));
                    ((RelayCommand)ReceiveCommand).RaiseCanExecuteChanged();
                }
            }
        }

        public bool HasTicketInput => !string.IsNullOrEmpty(_ticketInput);

        public string Status
        {
            get => _status;
            private set
            {
                if (_status != value)
                {
                    _status = value;
                    OnPropertyChanged(nameof(Status));
                    OnPropertyChanged(nameof(HasStatus));
                }
            }
        }

        public bool HasStatus => !string.IsNullOrEmpty(_status);

        public bool IsBusy
        {
            get => _isBusy;
            private set
            {
                if (_isBusy != value)
                {
                    _isBusy = value;
                    OnPropertyChanged(nameof(IsBusy));
                    OnPropertyChanged(nameof(FileStatusText));
                    ((RelayCommand)SendCommand).RaiseCanExecuteChanged();
                    ((RelayCommand)ReceiveCommand).RaiseCanExecuteChanged();
                }
            }
        }

        public bool IsError
        {
            get => _isError;
            private set
            {
                if (_isError != value)
                {
                    _isError = value;
                    OnPropertyChanged(nameof(IsError));
                    OnPropertyChanged(nameof(FileStatusText));
                }
            }
        }

        public string FileStatusText
        {
            get
            {
                if (IsBusy) return "In Progress";
                if (IsError) return "Failed";
                return "Completed";
            }
        }

        public string DestinationPath
        {
            get => _destinationPath;
            private set
            {
                if (_destinationPath != value)
                {
                    _destinationPath = value;
                    DestinationName = new System.IO.DirectoryInfo(value).Name;
                    OnPropertyChanged(nameof(DestinationPath));
                    OnPropertyChanged(nameof(HasDestinationPath));
                }
            }
        }

        public bool HasDestinationPath => !string.IsNullOrEmpty(_destinationPath);

        public string DestinationName
        {
            get => _destinationName;
            private set
            {
                if (_destinationName != value)
                {
                    _destinationName = value;
                    OnPropertyChanged(nameof(DestinationName));
                }
            }
        }

        public ICommand SendCommand { get; }
        public ICommand SelectSendFileCommand { get; }
        public ICommand ClearSendCommand { get; }
        public ICommand PasteTicketCommand { get; }
        public ICommand ClearTicketCommand { get; }
        public ICommand SelectDestinationCommand { get; }
        public ICommand ReceiveCommand { get; }
        public ICommand CopyTicketCommand { get; }
        public ICommand ShareTicketCommand { get; }

        public AppViewModel()
        {
            var dataDir = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "seyfr");
            Directory.CreateDirectory(dataDir);
            _core = new Core(dataDir);
            DestinationPath = Environment.GetFolderPath(Environment.SpecialFolder.MyDocuments);

            SelectSendFileCommand = new RelayCommand(() => _ = SelectSendFileAsync());
            SendCommand = new RelayCommand(async () => await SendAsync(), () => HasSelectedFile && !IsBusy);
            ClearSendCommand = new RelayCommand(ClearSend);
            PasteTicketCommand = new RelayCommand(PasteTicket);
            ClearTicketCommand = new RelayCommand(() => TicketInput = "");
            SelectDestinationCommand = new RelayCommand(() => _ = SelectDestinationAsync());
            ReceiveCommand = new RelayCommand(async () => await ReceiveAsync(), () => HasTicketInput && HasDestinationPath && !IsBusy);
            CopyTicketCommand = new RelayCommand(CopyTicket);
            ShareTicketCommand = new RelayCommand(ShareTicket);

        }


        private async Task SelectSendFileAsync()
        {
            var hwnd = WindowNative.GetWindowHandle(App.CurrentWindow);
            var picker = new FileOpenPicker();
            InitializeWithWindow.Initialize(picker, hwnd);
            picker.SuggestedStartLocation = PickerLocationId.DocumentsLibrary;
            picker.FileTypeFilter.Add("*");

            if (IsFolderMode)
            {
                var folderPicker = new FolderPicker();
                InitializeWithWindow.Initialize(folderPicker, hwnd);
                folderPicker.SuggestedStartLocation = PickerLocationId.DocumentsLibrary;
                folderPicker.FileTypeFilter.Add("*");
                var folder = await folderPicker.PickSingleFolderAsync();
                if (folder != null)
                {
                    SelectedFilePath = folder.Path;
                    SelectedFileName = folder.Name;
                    await SendAsync();
                }
            }
            else
            {
                var file = await picker.PickSingleFileAsync();
                if (file != null)
                {
                    SelectedFilePath = file.Path;
                    SelectedFileName = file.Name;
                    await SendAsync();
                }
            }
        }

        private async Task SendAsync()
        {
            if (string.IsNullOrEmpty(_selectedFilePath)) return;

            IsBusy = true;
            IsError = false;
            Status = "Sending...";

            try
            {
                var result = await Task.Run(() =>
                {
                    return _core.Send(_selectedFilePath!, null);
                });
                Ticket = result;
                if (!IsError)
                {
                    Status = "Ready to share";
                }
            }
            catch (SeyfrException ex)
            {
                IsError = true;
                Status = ex switch
                {
                    SeyfrException.Network e    => $"Network error: {e.details}",
                    SeyfrException.Io e         => $"File error: {e.details}",
                    SeyfrException.FileNotFound e => $"File not found: {e.path}",
                    SeyfrException.Store e      => $"Store error: {e.details}",
                    SeyfrException.Internal e   => $"Internal error: {e.details}",
                    SeyfrException.Timeout      => "Transfer timed out",
                    _                           => ex.Message
                };
            }
            catch (Exception ex)
            {
                IsError = true;
                var msg = ex.Message;
                if (string.IsNullOrWhiteSpace(msg) && ex.InnerException != null)
                    msg = ex.InnerException.Message;
                if (string.IsNullOrWhiteSpace(msg))
                    msg = ex.GetType().Name;
                Status = $"Error: {msg}";
            }
            finally
            {
                IsBusy = false;
            }
        }

        public void SetSendFile(string path, string name, bool isFolder)
        {
            IsFolderMode = isFolder;
            SelectedFilePath = path;
            SelectedFileName = name;
            _ = SendAsync();
        }

        private void ClearSend()
        {
            _selectedFilePath = null;
            SelectedFileName = null;
            Ticket = "";
            Status = "";
            IsError = false;
            OnPropertyChanged(nameof(TicketQrImage));
        }

        private async void PasteTicket()
        {
            var clipboard = Windows.ApplicationModel.DataTransfer.Clipboard.GetContent();
            if (clipboard.Contains(Windows.ApplicationModel.DataTransfer.StandardDataFormats.Text))
            {
                var text = await clipboard.GetTextAsync();
                TicketInput = text ?? "";
            }
        }


        private async Task SelectDestinationAsync()
        {
            var hwnd = WindowNative.GetWindowHandle(App.CurrentWindow);
            var picker = new FolderPicker();
            InitializeWithWindow.Initialize(picker, hwnd);
            picker.SuggestedStartLocation = PickerLocationId.DocumentsLibrary;
            picker.FileTypeFilter.Add("*");

            var folder = await picker.PickSingleFolderAsync();
            if (folder != null)
            {
                DestinationPath = folder.Path;
            }
        }

        private async Task ReceiveAsync()
        {
            if (string.IsNullOrEmpty(TicketInput) || string.IsNullOrEmpty(DestinationPath)) return;

            IsBusy = true;
            IsError = false;
            Status = "Receiving...";

            try
            {
                var ticket = TicketInput?.Trim() ?? "";
                await Task.Run(() =>
                {
                    _core.Receive(ticket, DestinationPath, null);
                });
                Status = "Received successfully";
            }
            catch (SeyfrException ex)
            {
                IsError = true;
                Status = ex switch
                {
                    SeyfrException.Network e      => $"Network error: {e.details}",
                    SeyfrException.Io e           => $"File error: {e.details}",
                    SeyfrException.InvalidTicket e => $"Invalid ticket: {e.details}",
                    SeyfrException.Store e        => $"Store error: {e.details}",
                    SeyfrException.Internal e     => $"Internal error: {e.details}",
                    SeyfrException.Cancelled      => "Transfer cancelled",
                    SeyfrException.Timeout        => "Transfer timed out",
                    _                             => ex.Message
                };
            }
            catch (Exception ex)
            {
                IsError = true;
                var msg = ex.Message;
                if (string.IsNullOrWhiteSpace(msg) && ex.InnerException != null)
                    msg = ex.InnerException.Message;
                if (string.IsNullOrWhiteSpace(msg))
                    msg = ex.GetType().Name;
                Status = $"Error: {msg}";
            }
            finally
            {
                IsBusy = false;
            }
        }

        private void CopyTicket()
        {
            if (string.IsNullOrEmpty(Ticket)) return;
            var package = new Windows.ApplicationModel.DataTransfer.DataPackage();
            package.SetText(Ticket);
            Windows.ApplicationModel.DataTransfer.Clipboard.SetContent(package);
            Status = "Copied to clipboard";
        }

        private void ShareTicket()
        {
            if (string.IsNullOrEmpty(Ticket)) return;
            var package = new Windows.ApplicationModel.DataTransfer.DataPackage();
            package.SetText(Ticket);
            Windows.ApplicationModel.DataTransfer.Clipboard.SetContent(package);
            Status = "Ticket shared to clipboard";
        }

        private void OnPropertyChanged(string propertyName)
        {
            PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
        }
    }

    /// <summary>
    /// Simple ICommand implementation for XAML binding.
    /// </summary>
    public class RelayCommand : ICommand
    {
        private readonly Action _execute;
        private readonly Func<bool>? _canExecute;

        public RelayCommand(Action execute, Func<bool>? canExecute = null)
        {
            _execute = execute;
            _canExecute = canExecute;
        }

        public event EventHandler? CanExecuteChanged;

        public bool CanExecute(object? parameter) => _canExecute?.Invoke() ?? true;

        public void Execute(object? parameter) => _execute();

        public void RaiseCanExecuteChanged() => CanExecuteChanged?.Invoke(this, EventArgs.Empty);
    }
}
