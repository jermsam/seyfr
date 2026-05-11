using Microsoft.UI.Xaml;
using System;
using uniffi.seyfr_core;
using Windows.ApplicationModel.DataTransfer;
using Windows.Storage;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Controls;

// To learn more about WinUI, the WinUI project structure,
// and more about our project templates, see: http://aka.ms/winui-project-info.

namespace Seyfr
{
    /// <summary>
    /// Main window that hosts the application UI with ViewModel-based data binding.
    /// </summary>
    public sealed partial class MainWindow : Window
    {
        public AppViewModel ViewModel { get; }

        public MainWindow()
        {
            this.InitializeComponent();
            ViewModel = new AppViewModel();
            RootGrid.DataContext = ViewModel;

            // Wire up navigation button clicks
            SendNavButton.Click += (s, e) => ViewModel.SelectedTab = TransferTab.Send;
            ReceiveNavButton.Click += (s, e) => ViewModel.SelectedTab = TransferTab.Receive;
        }

        private void BrowseButton_Click(object sender, RoutedEventArgs e)
        {
            ViewModel.SelectSendFileCommand.Execute(null);
        }

        private void DropArea_Tapped(object sender, TappedRoutedEventArgs e)
        {
            ViewModel.SelectSendFileCommand.Execute(null);
        }

        private void DropArea_DragEnter(object sender, DragEventArgs e)
        {
            DragOverlay.Opacity = 1;
        }

        private void DropArea_DragLeave(object sender, DragEventArgs e)
        {
            DragOverlay.Opacity = 0;
        }

        private void DropArea_DragOver(object sender, DragEventArgs e)
        {
            e.AcceptedOperation = DataPackageOperation.Copy;
            e.DragUIOverride.IsCaptionVisible = true;
            e.DragUIOverride.Caption = "Drop to send";
            e.Handled = true;
        }

        private void TicketInput_TextChanged(object sender, TextChangedEventArgs e)
        {
            if (sender is TextBox textBox)
            {
                ViewModel.TicketInput = textBox.Text;
            }
        }

        private async void DropArea_Drop(object sender, DragEventArgs e)
        {
            DragOverlay.Opacity = 0;
            if (e.DataView.Contains(StandardDataFormats.StorageItems))
            {
                var items = await e.DataView.GetStorageItemsAsync();
                if (items.Count > 0)
                {
                    var item = items[0];
                    bool isFolder = item is StorageFolder;
                    ViewModel.SetSendFile(item.Path, item.Name, isFolder);
                }
            }
        }
    }
}