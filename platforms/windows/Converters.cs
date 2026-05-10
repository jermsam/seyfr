using System;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Data;
using Microsoft.UI.Xaml.Media;

namespace Seyfr
{
    public class TabBackgroundConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, string language)
        {
            if (value is TransferTab tab && parameter is string param)
            {
                Enum.TryParse<TransferTab>(param, out var targetTab);
                if (tab == targetTab)
                {
                    var accent = GetAccentColor();
                    return new SolidColorBrush(
                        Microsoft.UI.ColorHelper.FromArgb(26, accent.R, accent.G, accent.B)
                    );
                }
            }
            return new SolidColorBrush(Microsoft.UI.Colors.Transparent);
        }

        public object ConvertBack(object value, Type targetType, object parameter, string language)
            => throw new NotImplementedException();

        private static Windows.UI.Color GetAccentColor()
        {
            try
            {
                return (Windows.UI.Color)Application.Current.Resources["SystemAccentColor"];
            }
            catch
            {
                return Microsoft.UI.ColorHelper.FromArgb(255, 0, 120, 212);
            }
        }
    }

    public class TabForegroundConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, string language)
        {
            if (value is TransferTab tab && parameter is string param)
            {
                Enum.TryParse<TransferTab>(param, out var targetTab);
                if (tab == targetTab)
                {
                    var accent = GetAccentColor();
                    return new SolidColorBrush(accent);
                }
            }
            return new SolidColorBrush(Microsoft.UI.ColorHelper.FromArgb(255, 150, 150, 150));
        }

        public object ConvertBack(object value, Type targetType, object parameter, string language)
            => throw new NotImplementedException();

        private static Windows.UI.Color GetAccentColor()
        {
            try
            {
                return (Windows.UI.Color)Application.Current.Resources["SystemAccentColor"];
            }
            catch
            {
                return Microsoft.UI.ColorHelper.FromArgb(255, 0, 120, 212);
            }
        }
    }

    public class IconBubbleBackgroundConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, string language)
        {
            if (value is TransferTab tab && parameter is string param)
            {
                Enum.TryParse<TransferTab>(param, out var targetTab);
                if (tab == targetTab)
                {
                    var accent = GetAccentColor();
                    return new SolidColorBrush(accent);
                }
            }
            return new SolidColorBrush(Microsoft.UI.ColorHelper.FromArgb(255, 150, 150, 150));
        }

        public object ConvertBack(object value, Type targetType, object parameter, string language)
            => throw new NotImplementedException();

        private static Windows.UI.Color GetAccentColor()
        {
            try
            {
                return (Windows.UI.Color)Application.Current.Resources["SystemAccentColor"];
            }
            catch
            {
                return Microsoft.UI.ColorHelper.FromArgb(255, 0, 120, 212);
            }
        }
    }

    public class TabTitleConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, string language)
        {
            if (value is TransferTab tab)
            {
                return tab == TransferTab.Send
                    ? "Send"
                    : "Receive";
            }

            return "";
        }

        public object ConvertBack(object value, Type targetType, object parameter, string language)
            => throw new NotImplementedException();
    }

    public class TabSubtitleConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, string language)
        {
            if (value is TransferTab tab)
            {
                return tab == TransferTab.Send
                    ? "Send your files to any device"
                    : "Receive files from any device";
            }

            return "";
        }

        public object ConvertBack(object value, Type targetType, object parameter, string language)
            => throw new NotImplementedException();
    }

    public class TabVisibilityConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, string language)
        {
            if (value is TransferTab tab && parameter is string param)
            {
                Enum.TryParse<TransferTab>(param, out var targetTab);

                return tab == targetTab
                    ? Visibility.Visible
                    : Visibility.Collapsed;
            }

            return Visibility.Collapsed;
        }

        public object ConvertBack(object value, Type targetType, object parameter, string language)
            => throw new NotImplementedException();
    }

    public class InverseBoolToVisibilityConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, string language)
        {
            if (value is bool b)
            {
                return b
                    ? Visibility.Collapsed
                    : Visibility.Visible;
            }

            return Visibility.Collapsed;
        }

        public object ConvertBack(object value, Type targetType, object parameter, string language)
            => throw new NotImplementedException();
    }

    public class BoolToFontWeightConverter : IValueConverter
    {
        public string TrueValue { get; set; } = "SemiBold";
        public string FalseValue { get; set; } = "Normal";

        public object Convert(object value, Type targetType, object parameter, string language)
        {
            if (value is bool b)
            {
                var weightStr = b ? TrueValue : FalseValue;

                return weightStr switch
                {
                    "Light" => Microsoft.UI.Text.FontWeights.Light,
                    "Normal" => Microsoft.UI.Text.FontWeights.Normal,
                    "SemiBold" => Microsoft.UI.Text.FontWeights.SemiBold,
                    "Bold" => Microsoft.UI.Text.FontWeights.Bold,
                    _ => Microsoft.UI.Text.FontWeights.Normal,
                };
            }

            return Microsoft.UI.Text.FontWeights.Normal;
        }

        public object ConvertBack(object value, Type targetType, object parameter, string language)
            => throw new NotImplementedException();
    }

    public class FolderModeGlyphConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, string language)
        {
            if (value is bool isFolderMode && isFolderMode)
            {
                // Folder icon
                return "\uE8B7";
            }

            // File icon
            return "\uE7C3";
        }

        public object ConvertBack(object value, Type targetType, object parameter, string language)
            => throw new NotImplementedException();
    }

    public class TabFontWeightConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, string language)
        {
            if (value is TransferTab tab && parameter is string param)
            {
                Enum.TryParse<TransferTab>(param, out var targetTab);
                return tab == targetTab
                    ? Microsoft.UI.Text.FontWeights.SemiBold
                    : Microsoft.UI.Text.FontWeights.Medium;
            }
            return Microsoft.UI.Text.FontWeights.Medium;
        }

        public object ConvertBack(object value, Type targetType, object parameter, string language)
            => throw new NotImplementedException();
    }

    public class TabStatusTextConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, string language)
        {
            if (value is TransferTab tab)
            {
                return tab == TransferTab.Send
                    ? "Ready to send files"
                    : "Ready to receive files";
            }
            return "Ready to send files";
        }

        public object ConvertBack(object value, Type targetType, object parameter, string language)
            => throw new NotImplementedException();
    }

    public class BoolToOpacityConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, string language)
        {
            if (value is bool b)
                return b ? 1.0 : 0.5;
            return 0.5;
        }

        public object ConvertBack(object value, Type targetType, object parameter, string language)
            => throw new NotImplementedException();
    }

    public class InverseBoolToOpacityConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, string language)
        {
            if (value is bool b)
                return b ? 0.5 : 1.0;
            return 1.0;
        }

        public object ConvertBack(object value, Type targetType, object parameter, string language)
            => throw new NotImplementedException();
    }
}