import gi
import os
import threading
import qrcode
from io import BytesIO
from PIL import Image

gi.require_version('Gtk', '4.0')
gi.require_version('Adw', '1')
from gi.repository import Gtk, Adw, Gdk, Gio, GLib, GdkPixbuf

from core_wrapper import CoreWrapper

class SeyfrWindow(Adw.ApplicationWindow):
    def __init__(self, **kwargs):
        super().__init__(**kwargs)
        
        self.core = CoreWrapper()
        self.selected_tab = "send"
        self.is_folder_mode = False
        self.selected_file_path = None
        self.current_ticket = None
        
        self.set_title("Seyfr")
        self.set_default_size(940, 640)
        
        # Load CSS
        self.load_css()
        
        # Main Layout: Modern Navigation Split View (GNOME 46+)
        self.split_view = Adw.NavigationSplitView()
        
        # Sidebar
        self.create_sidebar()
        
        # Content
        self.create_content()
        
        self.set_content(self.split_view)
        
        # Initial State
        self.update_view()

    def load_css(self):
        css_provider = Gtk.CssProvider()
        css_path = os.path.join(os.path.dirname(__file__), "style.css")
        if os.path.exists(css_path):
            css_provider.load_from_path(css_path)
            Gtk.StyleContext.add_provider_for_display(
                Gdk.Display.get_default(),
                css_provider,
                Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION
            )

    def create_sidebar(self):
        sidebar_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        sidebar_box.add_css_class("sidebar")
        
        # Logo Section
        logo_container = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12)
        logo_container.set_margin_top(32)
        logo_container.set_margin_bottom(32)
        logo_container.set_margin_start(20)
        logo_container.set_margin_end(20)

        logo_label = Gtk.Label(label="S")
        logo_label.add_css_class("logo-icon")
        logo_container.append(logo_label)
        
        brand_name = Gtk.Label(label="Seyfr")
        brand_name.add_css_class("brand-name")
        logo_container.append(brand_name)
        
        sidebar_box.append(logo_container)

        # Navigation
        self.nav_list = Gtk.ListBox()
        self.nav_list.add_css_class("navigation-sidebar")
        self.nav_list.connect("row-selected", self.on_nav_row_selected)

        self.send_row = self.create_nav_row("Send", "mail-send-symbolic")
        self.receive_row = self.create_nav_row("Receive", "mail-receive-symbolic")
        
        self.nav_list.append(self.send_row)
        self.nav_list.append(self.receive_row)
        sidebar_box.append(self.nav_list)

        sidebar_page = Adw.NavigationPage.new(sidebar_box, "Sidebar")
        self.split_view.set_sidebar(sidebar_page)

    def create_nav_row(self, title, icon_name):
        row = Gtk.ListBoxRow()
        box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=12)
        box.set_margin_start(16)
        box.set_margin_end(16)
        box.set_margin_top(10)
        box.set_margin_bottom(10)
        icon = Gtk.Image.new_from_icon_name(icon_name)
        box.append(icon)
        label = Gtk.Label(label=title)
        box.append(label)
        row.set_child(box)
        row.tag = title.lower()
        return row

    def on_nav_row_selected(self, listbox, row):
        if row:
            self.selected_tab = row.tag
            self.update_view()

    def create_content(self):
        self.stack = Adw.ViewStack()
        
        # Pages
        self.setup_send_page()
        self.setup_receive_page()

        # Toolbar View (Modern Adwaita)
        toolbar_view = Adw.ToolbarView()
        header = Adw.HeaderBar()
        toolbar_view.add_top_bar(header)
        toolbar_view.set_content(self.stack)

        content_page = Adw.NavigationPage.new(toolbar_view, "Content")
        self.split_view.set_content(content_page)

    def setup_send_page(self):
        page = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=24)
        page.set_margin_top(40)
        page.set_margin_start(40)
        page.set_margin_end(40)
        
        self.drop_zone = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12)
        self.drop_zone.add_css_class("drop-zone")
        self.drop_zone.set_vexpand(True)
        self.drop_zone.set_valign(Gtk.Align.CENTER)
        
        drop_icon = Gtk.Image.new_from_icon_name("document-send-symbolic")
        drop_icon.set_pixel_size(64)
        self.drop_zone.append(drop_icon)
        
        label = Gtk.Label(label="Drag & Drop files to start")
        label.add_css_class("title-1")
        self.drop_zone.append(label)
        
        page.append(self.drop_zone)
        self.stack.add_titled(page, "send", "Send")

    def setup_receive_page(self):
        page = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=24)
        page.set_valign(Gtk.Align.CENTER)
        page.set_margin_start(60)
        page.set_margin_end(60)
        
        title = Gtk.Label(label="Receive Files")
        title.add_css_class("title-1")
        page.append(title)
        
        self.ticket_entry = Gtk.Entry()
        self.ticket_entry.set_placeholder_text("Enter ticket ID...")
        page.append(self.ticket_entry)
        
        receive_btn = Gtk.Button(label="Start Download")
        receive_btn.add_css_class("suggested-action")
        receive_btn.add_css_class("pill")
        page.append(receive_btn)
        
        self.stack.add_titled(page, "receive", "Receive")

    def update_view(self):
        self.stack.set_visible_child_name(self.selected_tab)
