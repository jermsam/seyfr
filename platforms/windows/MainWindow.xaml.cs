using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Input;
using System;
using Windows.ApplicationModel.DataTransfer;
using Windows.Storage;

namespace Seyfr
{
    public sealed partial class MainWindow : Window
    {
        public AppViewModel ViewModel { get; }

        public MainWindow()
        {
            this.InitializeComponent();
            ViewModel = new AppViewModel();

            // Wire up navigation button clicks
            SendNavButton.Click += (s, e) => ViewModel.SelectedTab = TransferTab.Send;
            ReceiveNavButton.Click += (s, e) => ViewModel.SelectedTab = TransferTab.Receive;
        }

        private void BrowseButton_Click(object sender, RoutedEventArgs e)
        {
            ViewModel.SelectSendFileCommand.Execute(null);
        }

        private void DropArea_DragOver(object sender, DragEventArgs e)
        {
            e.AcceptedOperation = DataPackageOperation.Copy;
            e.DragUIOverride.IsCaptionVisible = true;
            e.DragUIOverride.Caption = "Drop to send";
        }

        private async void DropArea_Drop(object sender, DragEventArgs e)
        {
            if (e.DataView.Contains(StandardDataFormats.StorageItems))
            {
                var items = await e.DataView.GetStorageItemsAsync();
                if (items.Count > 0)
                {
                    var item = items[0];
                    if (item is StorageFile file)
                    {
                        ViewModel.SetSendFile(file.Path, file.Name);
                    }
                    else if (item is StorageFolder folder)
                    {
                        ViewModel.SetSendFile(folder.Path, folder.Name);
                    }
                }
            }
        }
    }
}
