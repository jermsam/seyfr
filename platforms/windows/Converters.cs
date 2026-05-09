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
                    return Application.Current.Resources["AccentFillColorDefaultBrush"];
                }
            }
            return Application.Current.Resources["Transparent"];
        }

        public object ConvertBack(object value, Type targetType, object parameter, string language)
            => throw new NotImplementedException();
    }

    public class TabTitleConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, string language)
        {
            if (value is TransferTab tab)
            {
                return tab == TransferTab.Send ? "Send" : "Receive";
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
                return tab == targetTab ? Visibility.Visible : Visibility.Collapsed;
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
                return b ? Visibility.Collapsed : Visibility.Visible;
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
}
