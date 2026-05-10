using Microsoft.UI.Input;
using Microsoft.UI.Xaml.Controls;

namespace Seyfr
{
    /// <summary>
    /// A Button that shows a hand cursor when the pointer hovers over it.
    /// Uses the official WinUI 3 ProtectedCursor API.
    /// </summary>
    public class HandCursorButton : Button
    {
        public HandCursorButton()
        {
            this.ProtectedCursor = InputSystemCursor.Create(InputSystemCursorShape.Hand);
        }
    }
}
