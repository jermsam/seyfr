using Microsoft.UI.Xaml.Media.Imaging;
using QRCoder;
using System.Runtime.InteropServices;

namespace Seyfr
{
    /// <summary>
    /// Generates a WinUI 3 WriteableBitmap QR code from a text string using QRCoder.
    /// Uses IMemoryBufferByteAccess to write raw pixels directly to the buffer.
    /// </summary>
    public static class QrCodeHelper
    {
        [ComImport]
        [Guid("5B0D3235-4DBA-4D44-865E-8F1D0E4FD04D")]
        [InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
        unsafe interface IMemoryBufferByteAccess
        {
            void GetBuffer(out byte* buffer, out uint capacity);
        }

        public static unsafe WriteableBitmap Generate(string text, int pixelsPerModule = 8)
        {
            var qrGenerator = new QRCodeGenerator();
            var qrCodeData = qrGenerator.CreateQrCode(text, QRCodeGenerator.ECCLevel.Q);
            int moduleCount = qrCodeData.ModuleMatrix.Count;
            int size = moduleCount * pixelsPerModule;

            var writeableBitmap = new WriteableBitmap(size, size);

            using (var reference = writeableBitmap.PixelBuffer.CreateReference())
            {
                byte* data;
                uint capacity;
                ((IMemoryBufferByteAccess)reference).GetBuffer(out data, out capacity);

                for (int y = 0; y < size; y++)
                {
                    for (int x = 0; x < size; x++)
                    {
                        int moduleX = x / pixelsPerModule;
                        int moduleY = y / pixelsPerModule;
                        bool isBlack = qrCodeData.ModuleMatrix[moduleY][moduleX];

                        int index = (y * size + x) * 4;
                        byte color = isBlack ? (byte)0 : (byte)255;
                        data[index] = color;     // B
                        data[index + 1] = color; // G
                        data[index + 2] = color; // R
                        data[index + 3] = 255;   // A
                    }
                }
            }

            return writeableBitmap;
        }
    }
}
