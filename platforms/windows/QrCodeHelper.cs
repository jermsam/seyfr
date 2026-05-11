using Microsoft.UI.Xaml.Media.Imaging;
using QRCoder;
using System.Runtime.InteropServices;
using Windows.Storage.Streams;

namespace Seyfr
{
    /// <summary>
    /// Generates a WinUI 3 WriteableBitmap QR code from a text string using QRCoder.
    /// Uses IBufferByteAccess COM interface to write raw pixels directly to the buffer.
    /// </summary>
    public static class QrCodeHelper
    {
        [ComImport]
        [Guid("905a0fef-bc53-11df-8c49-001e4fc686da")]
        [InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
        unsafe interface IBufferByteAccess
        {
            void Buffer(out byte* value);
        }

        public static unsafe WriteableBitmap Generate(string text, int pixelsPerModule = 8)
        {
            var qrGenerator = new QRCodeGenerator();
            var qrCodeData = qrGenerator.CreateQrCode(text, QRCodeGenerator.ECCLevel.M);
            int moduleCount = qrCodeData.ModuleMatrix.Count;
            int size = moduleCount * pixelsPerModule;

            var writeableBitmap = new WriteableBitmap(size, size);
            IBuffer buffer = writeableBitmap.PixelBuffer;

            byte* data;
            ((IBufferByteAccess)buffer).Buffer(out data);

            for (int y = 0; y < size; y++)
            {
                int moduleY = y / pixelsPerModule;
                for (int x = 0; x < size; x++)
                {
                    int moduleX = x / pixelsPerModule;
                    bool isBlack = qrCodeData.ModuleMatrix[moduleY][moduleX];

                    int index = (y * size + x) * 4;
                    byte color = isBlack ? (byte)0 : (byte)255;
                    data[index] = color;     // B
                    data[index + 1] = color; // G
                    data[index + 2] = color; // R
                    data[index + 3] = 255;   // A
                }
            }

            writeableBitmap.Invalidate();
            return writeableBitmap;
        }
    }
}
