import gi

gi.require_version('Gtk', '4.0')
gi.require_version('Adw', '1')
from gi.repository import Gtk, Adw, Gio

from window import SeyfrWindow

class SeyfrApplication(Adw.Application):
    def __init__(self):
        super().__init__(
            application_id='com.jitpomi.seyfr',
            flags=Gio.ApplicationFlags.DEFAULT_FLAGS
        )
        self.window = None

    def do_activate(self):
        if not self.window:
            self.window = SeyfrWindow(application=self)
        self.window.present()