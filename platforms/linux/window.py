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
        self.set_default_size(900, 600)
        self.set_title("Seyfr")

        # Core initialization
        self.core = CoreWrapper()
        
        # Load CSS
        self.load_css()
        
        # Main Layout using a horizontal box for sidebar + content
        # This replaces Adw.NavigationSplitView for Libadwaita 1.2 compatibility
        self.main_layout = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL)
        self.set_content(self.main_layout)

        # 1. Sidebar Setup
        self.sidebar_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self.sidebar_box.add_css_class("sidebar")
        self.sidebar_box.set_size_request(240, -1)
        
        # Branding
        brand_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=8)
        brand_box.set_margin_top(32)
        brand_box.set_margin_bottom(32)
        brand_box.set_margin_start(20)
        brand_box.set_margin_end(20)
        
        logo_label = Gtk.Label(label="S")
        logo_label.add_css_class("logo-icon")
        brand_box.append(logo_label)
        
        brand_name = Gtk.Label(label="Seyfr")
        brand_name.add_css_class("brand-name")
        brand_box.append(brand_name)
        
        self.sidebar_box.append(brand_box)
        
        # Navigation List
        self.nav_list = Gtk.ListBox()
        self.nav_list.add_css_class("navigation-sidebar")
        self.nav_list.set_selection_mode(Gtk.SelectionMode.SINGLE)
        self.nav_list.connect("row-selected", self.on_nav_changed)
        
        self.send_row = self.create_nav_row("Send", "mail-send-symbolic")
        self.receive_row = self.create_nav_row("Receive", "mail-receive-symbolic")
        
        self.nav_list.append(self.send_row)
        self.nav_list.append(self.receive_row)
        
        self.sidebar_box.append(self.nav_list)
        
        # Spacer
        spacer = Gtk.Box()
        spacer.set_vexpand(True)
        self.sidebar_box.append(spacer)
        
        # Node ID Badge
        self.node_id_label = Gtk.Label(label=f"Node: {self.core.node_id[:8]}...")
        self.node_id_label.add_css_class("node-id-badge")
        self.sidebar_box.append(self.node_id_label)
        
        self.main_layout.append(self.sidebar_box)

        # 2. Content Area Setup
        self.content_stack = Gtk.Stack()
        self.content_stack.set_transition_type(Gtk.StackTransitionType.CROSSFADE)
        self.content_stack.set_hexpand(True)
        
        # Wrapper for content with a HeaderBar
        self.content_wrapper = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)
        self.header = Adw.HeaderBar()
        self.header.set_show_end_title_buttons(True)
        self.content_wrapper.append(self.header)
        self.content_wrapper.append(self.content_stack)
        
        self.main_layout.append(self.content_wrapper)
        
        # Initialize Pages
        self.setup_send_page()
        self.setup_receive_page()
        
        # Select first row
        self.nav_list.select_row(self.send_row)

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
        row.title = title
        return row

    def on_nav_changed(self, listbox, row):
        if not row: return
        self.content_stack.set_visible_child_name(row.title)
        self.header.set_title(row.title)

    def setup_send_page(self):
        page = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=24)
        page.set_margin_top(40)
        page.set_margin_start(40)
        page.set_margin_end(40)
        page.set_margin_bottom(40)
        
        # 1. Drop Zone
        self.drop_zone = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12)
        self.drop_zone.add_css_class("drop-zone")
        self.drop_zone.set_vexpand(True)
        self.drop_zone.set_valign(Gtk.Align.CENTER)
        
        drop_icon = Gtk.Image.new_from_icon_name("document-send-symbolic")
        drop_icon.set_pixel_size(64)
        self.drop_zone.append(drop_icon)
        
        self.drop_label = Gtk.Label(label="Drag files here to send")
        self.drop_label.add_css_class("drop-title")
        self.drop_zone.append(self.drop_label)
        
        subtitle = Gtk.Label(label="Files will be shared securely via Seyfr core")
        subtitle.add_css_class("drop-subtitle")
        self.drop_zone.append(subtitle)
        
        # Drag and Drop support
        target = Gtk.DropTarget.new(Gio.ListStore.new(Gdk.FileList), Gdk.DragAction.COPY)
        target.connect("drop", self.on_file_drop)
        self.drop_zone.add_controller(target)
        
        page.append(self.drop_zone)
        
        # 2. Controls
        controls = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=12)
        controls.set_halign(Gtk.Align.CENTER)
        
        self.file_toggle = Gtk.ToggleButton(label="File")
        self.file_toggle.set_active(True)
        self.folder_toggle = Gtk.ToggleButton(label="Folder")
        self.folder_toggle.set_group(self.file_toggle)
        
        controls.append(self.file_toggle)
        controls.append(self.folder_toggle)
        
        page.append(controls)
        
        # 3. QR Code Area (Hidden initially)
        self.qr_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=16)
        self.qr_box.set_visible(False)
        
        self.qr_image = Gtk.Image()
        self.qr_image.set_pixel_size(256)
        self.qr_box.append(self.qr_image)
        
        self.ticket_label = Gtk.Label(label="Ticket ID: ...")
        self.ticket_label.add_css_class("ticket-id")
        self.qr_box.append(self.ticket_label)
        
        page.append(self.qr_box)
        
        self.content_stack.add_named(page, "Send")

    def setup_receive_page(self):
        page = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=32)
        page.set_valign(Gtk.Align.CENTER)
        page.set_margin_start(80)
        page.set_margin_end(80)
        
        title = Gtk.Label(label="Receive Files")
        title.add_css_class("page-title")
        page.append(title)
        
        # Input Section
        input_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12)
        
        label = Gtk.Label(label="Enter Transfer Ticket")
        label.set_halign(Gtk.Align.START)
        input_box.append(label)
        
        self.ticket_entry = Gtk.Entry()
        self.ticket_entry.set_placeholder_text("Paste ticket here...")
        self.ticket_entry.add_css_class("ticket-entry")
        input_box.append(self.ticket_entry)
        
        page.append(input_box)
        
        # Receive Button
        self.receive_btn = Gtk.Button(label="Receive")
        self.receive_btn.add_css_class("suggested-action")
        self.receive_btn.add_css_class("pill")
        self.receive_btn.set_size_request(200, 50)
        self.receive_btn.set_halign(Gtk.Align.CENTER)
        self.receive_btn.connect("clicked", self.on_receive_clicked)
        page.append(self.receive_btn)
        
        self.content_stack.add_named(page, "Receive")

    def on_file_drop(self, target, value, x, y):
        if isinstance(value, Gdk.FileList):
            files = value.get_files()
            if files:
                path = files[0].get_path()
                self.start_send(path)
                return True
        return False

    def start_send(self, path):
        self.drop_zone.set_visible(False)
        self.qr_box.set_visible(True)
        
        # Run in background thread
        thread = threading.Thread(target=self._send_thread, args=(path,))
        thread.daemon = True
        thread.start()

    def _send_thread(self, path):
        try:
            is_directory = self.folder_toggle.get_active()
            ticket = self.core.send(path, is_directory)
            GLib.idle_add(self.update_qr, ticket)
        except Exception as e:
            print(f"Send error: {e}")

    def update_qr(self, ticket):
        self.ticket_label.set_label(f"Ticket: {ticket[:12]}...")
        pixbuf = self.generate_qr(ticket)
        self.qr_image.set_from_pixbuf(pixbuf)

    def generate_qr(self, data):
        qr = qrcode.QRCode(version=1, box_size=10, border=4)
        qr.add_data(data)
        qr.make(fit=True)
        
        img = qr.make_image(fill_color="black", back_color="white")
        
        # Convert PIL to Pixbuf
        import io
        buf = io.BytesIO()
        img.save(buf, format="PNG")
        loader = GdkPixbuf.PixbufLoader.new_with_type("png")
        loader.write(buf.getvalue())
        loader.close()
        return loader.get_pixbuf()

    def on_receive_clicked(self, btn):
        ticket = self.ticket_entry.get_text().strip()
        if not ticket: return
        
        btn.set_sensitive(False)
        thread = threading.Thread(target=self._receive_thread, args=(ticket,))
        thread.daemon = True
        thread.start()

    def _receive_thread(self, ticket):
        try:
            # For demo, we receive to Downloads
            import os
            dest = os.path.expanduser("~/Downloads")
            self.core.receive(ticket, dest)
            GLib.idle_add(self.on_receive_complete)
        except Exception as e:
            print(f"Receive error: {e}")
            GLib.idle_add(lambda: self.receive_btn.set_sensitive(True))

    def on_receive_complete(self):
        self.receive_btn.set_sensitive(True)
        self.ticket_entry.set_text("")
        # Show a notification or dialog here
        print("Transfer complete!")
