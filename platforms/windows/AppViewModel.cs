using System;
using System.ComponentModel;
using System.ComponentModel.Design;
using System.IO;
using System.Threading.Tasks;
using System.Windows.Input;
using uniffi.seyfr_core;
using WinRT.Interop;
using Windows.Storage.Pickers;

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
        private string _destinationPath = "";

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
                }
            }
        }

        public bool HasTicket => !string.IsNullOrEmpty(_ticket);

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
                    ((RelayCommand)SendCommand).RaiseCanExecuteChanged();
                    ((RelayCommand)ReceiveCommand).RaiseCanExecuteChanged();
                }
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
                    OnPropertyChanged(nameof(DestinationPath));
                    OnPropertyChanged(nameof(HasDestinationPath));
                }
            }
        }

        public bool HasDestinationPath => !string.IsNullOrEmpty(_destinationPath);


        public ICommand SendCommand { get; }
        public ICommand SelectSendFileCommand { get; }
        public ICommand ClearSendCommand { get; }
        public ICommand PasteTicketCommand { get; }
        public ICommand ClearTicketCommand { get; }
        public ICommand SelectDestinationCommand { get; }
        public ICommand ReceiveCommand { get; }
        public ICommand CopyTicketCommand { get; }

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
                    _selectedFilePath = folder.Path;
                    SelectedFileName = folder.Name;
                }
            }
            else
            {
                var file = await picker.PickSingleFileAsync();
                if (file != null)
                {
                    _selectedFilePath = file.Path;
                    SelectedFileName = file.Name;
                }
            }
        }

        private async Task SendAsync()
        {
            if (string.IsNullOrEmpty(_selectedFilePath)) return;

            IsBusy = true;
            Status = "Sending...";

            try
            {
                await Task.Run(() =>
                {
                    var result = _core.Send(_selectedFilePath!, null);
                    Ticket = result;
                });
                Status = "File sent! Ticket copied.";
            }
            catch (Exception ex)
            {
                Status = $"Error: {ex.Message}";
            }
            finally
            {
                IsBusy = false;
            }
        }

        public void SetSendFile(string path, string name)
        {
            _selectedFilePath = path;
            SelectedFileName = name;
        }

        private void ClearSend()
        {
            _selectedFilePath = null;
            SelectedFileName = null;
            Ticket = "";
            Status = "";
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
            Status = "Receiving...";

            try
            {
                await Task.Run(() =>
                {
                    _core.Receive(TicketInput, DestinationPath, null);
                });
                Status = "File received successfully!";
            }
            catch (Exception ex)
            {
                Status = $"Error: {ex.Message}";
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
