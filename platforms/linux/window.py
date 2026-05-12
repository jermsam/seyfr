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
        self.set_default_size(1400, 950)
        
        # Load CSS
        self.load_css()
        
        # Main Layout: Compatible Horizontal Box
        self.main_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=0)
        
        # Sidebar
        self.create_sidebar()
        
        # Content Stack
        self.content_stack = Gtk.Stack()
        self.content_stack.set_transition_type(Gtk.StackTransitionType.CROSSFADE)
        self.content_stack.set_hexpand(True)
        self.content_stack.set_vexpand(True)
        
        self.create_content()
        
        self.main_box.append(self.sidebar_box)
        self.main_box.append(self.content_stack)
        
        self.set_content(self.main_box)
        
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
        self.sidebar_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self.sidebar_box.set_size_request(260, -1)
        self.sidebar_box.add_css_class("sidebar")
        
        # Scrollable content part
        content_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        content_box.set_vexpand(True)
        
        # Logo Section
        logo_container = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=4)
        logo_container.set_margin_top(48)
        logo_container.set_margin_bottom(32)
        logo_container.set_margin_start(24)
        logo_container.set_margin_end(24)
        logo_container.set_halign(Gtk.Align.START)

        logo_label = Gtk.Label(label="S")
        logo_label.add_css_class("logo-icon")
        logo_container.append(logo_label)
        
        brand_name = Gtk.Label(label="SEYFR")
        brand_name.add_css_class("brand-name")
        logo_container.append(brand_name)
        
        subtitle = Gtk.Label(label="Send Your Files Right")
        subtitle.add_css_class("dim-label")
        logo_container.append(subtitle)
        
        content_box.append(logo_container)

        # Navigation
        self.nav_list = Gtk.ListBox()
        self.nav_list.add_css_class("navigation-sidebar")
        self.nav_list.connect("row-selected", self.on_nav_row_selected)

        self.send_row = self.create_nav_row("Send", "mail-send-symbolic")
        self.receive_row = self.create_nav_row("Receive", "mail-receive-symbolic")
        
        self.nav_list.append(self.send_row)
        self.nav_list.append(self.receive_row)
        content_box.append(self.nav_list)
        
        self.sidebar_box.append(content_box)
        
        # Status Indicator at bottom
        status_container = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=4)
        status_container.add_css_class("status-container")
        
        dot_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=0)
        status_dot = Gtk.Box()
        status_dot.add_css_class("status-dot")
        dot_box.append(status_dot)
        
        status_label = Gtk.Label(label="Online")
        status_label.add_css_class("status-label")
        dot_box.append(status_label)
        status_container.append(dot_box)
        
        sub_status = Gtk.Label(label="Ready to send files")
        sub_status.add_css_class("status-sublabel")
        sub_status.set_halign(Gtk.Align.START)
        status_container.append(sub_status)
        
        self.sidebar_box.append(status_container)

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
        row.id = title.lower()
        return row

    def create_content(self):
        # Send Page
        self.send_page = self.create_send_page()
        self.content_stack.add_named(self.send_page, "send")
        
        # Receive Page
        self.receive_page = self.create_receive_page()
        self.content_stack.add_named(self.receive_page, "receive")

    def create_send_page(self):
        page = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        
        # Header (Compatible HeaderBar)
        header = Gtk.HeaderBar()
        header.add_css_class("flat")
        page.append(header)
        
        scrolled = Gtk.ScrolledWindow()
        scrolled.set_vexpand(True)
        
        container = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=32)
        container.set_margin_start(40)
        container.set_margin_end(40)
        container.set_margin_top(40)
        container.set_margin_bottom(40)
        container.set_halign(Gtk.Align.CENTER)
        container.set_valign(Gtk.Align.START)
        container.set_size_request(600, -1)
        
        title_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=8)
        title = Gtk.Label(label="Send")
        title.add_css_class("page-title")
        title_box.append(title)
        
        subtitle = Gtk.Label(label="Send your files to any device")
        subtitle.add_css_class("dim-label")
        title_box.append(subtitle)
        container.append(title_box)
        
        # Drop Zone with Concentric Rings
        self.drop_zone = Gtk.Button()
        self.drop_zone.add_css_class("drop-zone")
        self.drop_zone.connect("clicked", self.on_select_file_clicked)
        
        overlay = Gtk.Overlay()
        
        # Rings Container
        rings_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        rings_box.add_css_class("concentric-container")
        rings_box.set_halign(Gtk.Align.CENTER)
        rings_box.set_valign(Gtk.Align.CENTER)
        
        # Add 6 concentric rings with better spacing
        for i in range(6):
            ring = Gtk.Box()
            ring.add_css_class("concentric-ring")
            # Increase size decrement for better spacing
            size = 320 - (i * 45)
            ring.set_size_request(size, size)
            ring.set_halign(Gtk.Align.CENTER)
            ring.set_valign(Gtk.Align.CENTER)
            if i == 0:
                rings_box.append(ring)
                current_parent = ring
            else:
                current_parent.append(ring)
                current_parent = ring
        
        # Icon in center
        center_icon = Gtk.Image.new_from_icon_name("document-send-symbolic")
        center_icon.set_pixel_size(32)
        center_icon.set_halign(Gtk.Align.CENTER)
        center_icon.set_valign(Gtk.Align.CENTER)
        current_parent.append(center_icon)
        
        overlay.set_child(rings_box)
        self.drop_zone.set_child(overlay)
        container.append(self.drop_zone)
        
        # Drag & Drop Label
        labels_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=4)
        dd_label = Gtk.Label(label="Drag & drop files here")
        dd_label.add_css_class("status-label")
        labels_box.append(dd_label)
        
        browse_label = Gtk.Label(label="or click to browse")
        browse_label.add_css_class("dim-label")
        labels_box.append(browse_label)
        container.append(labels_box)
        
        # Mode Toggle
        mode_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=12)
        mode_box.set_halign(Gtk.Align.CENTER)
        
        file_label = Gtk.Label(label="File mode")
        file_label.add_css_class("dim-label")
        mode_box.append(file_label)
        
        self.mode_switch = Gtk.Switch()
        self.mode_switch.connect("notify::active", self.on_mode_toggled)
        mode_box.append(self.mode_switch)
        
        folder_label = Gtk.Label(label="Folder mode")
        folder_label.add_css_class("dim-label")
        mode_box.append(folder_label)
        container.append(mode_box)
        
        # Send Button
        self.send_button = Gtk.Button(label="Generate Ticket")
        self.send_button.add_css_class("pill")
        self.send_button.add_css_class("suggested-action")
        self.send_button.set_sensitive(False)
        self.send_button.connect("clicked", self.on_send_clicked)
        container.append(self.send_button)
        
        # Ticket Display
        self.ticket_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=20)
        self.ticket_box.add_css_class("ticket-box")
        self.ticket_box.set_visible(False)
        self.ticket_box.set_margin_top(24)
        
        ticket_label = Gtk.Label(label="YOUR SECURE TICKET")
        ticket_label.add_css_class("dim-label")
        self.ticket_box.append(ticket_label)
        
        self.ticket_entry = Gtk.Entry()
        self.ticket_entry.add_css_class("ticket-entry")
        self.ticket_entry.set_editable(False)
        self.ticket_box.append(self.ticket_entry)
        
        qr_container = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        qr_container.set_halign(Gtk.Align.CENTER)
        self.qr_image = Gtk.Image()
        self.qr_image.add_css_class("qr-image")
        qr_container.append(self.qr_image)
        self.ticket_box.append(qr_container)
        
        container.append(self.ticket_box)
        
        scrolled.set_child(container)
        page.append(scrolled)
        return page

    def create_receive_page(self):
        page = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        
        header = Gtk.HeaderBar()
        header.add_css_class("flat")
        page.append(header)
        
        container = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=32)
        container.set_margin_start(40)
        container.set_margin_end(40)
        container.set_margin_top(40)
        container.set_halign(Gtk.Align.CENTER)
        container.set_size_request(600, -1)
        
        title_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=8)
        title = Gtk.Label(label="Receive")
        title.add_css_class("page-title")
        title_box.append(title)
        
        subtitle = Gtk.Label(label="Receive files from any device")
        subtitle.add_css_class("dim-label")
        title_box.append(subtitle)
        container.append(title_box)
        
        # Enter Ticket Card
        ticket_card = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12)
        ticket_card.add_css_class("section-card")
        
        ticket_header = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=0)
        ticket_label = Gtk.Label(label="Enter ticket")
        ticket_label.add_css_class("status-label")
        ticket_header.append(ticket_label)
        
        paste_btn = Gtk.Button(label="Paste")
        paste_btn.set_halign(Gtk.Align.END)
        paste_btn.set_hexpand(True)
        paste_btn.connect("clicked", self.on_paste_clicked)
        ticket_header.append(paste_btn)
        ticket_card.append(ticket_header)
        
        self.receive_entry = Gtk.Entry()
        self.receive_entry.set_placeholder_text("Paste ticket here...")
        ticket_card.append(self.receive_entry)
        container.append(ticket_card)
        
        # Save Location Card
        save_card = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12)
        save_card.add_css_class("section-card")
        
        save_header = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=0)
        save_label = Gtk.Label(label="Save Location")
        save_label.add_css_class("status-label")
        save_header.append(save_label)
        
        change_btn = Gtk.Button(label="Change")
        change_btn.set_halign(Gtk.Align.END)
        change_btn.set_hexpand(True)
        save_header.append(change_btn)
        save_card.append(save_header)
        
        loc_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=12)
        loc_icon = Gtk.Image.new_from_icon_name("folder-symbolic")
        loc_box.append(loc_icon)
        
        loc_info = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=2)
        loc_name = Gtk.Label(label="Downloads")
        loc_name.set_halign(Gtk.Align.START)
        loc_info.append(loc_name)
        
        loc_path = Gtk.Label(label="~/Downloads")
        loc_path.add_css_class("dim-label")
        loc_path.set_halign(Gtk.Align.START)
        loc_info.append(loc_path)
        loc_box.append(loc_info)
        save_card.append(loc_box)
        container.append(save_card)
        
        self.receive_button = Gtk.Button(label="Receive File")
        self.receive_button.add_css_class("pill")
        self.receive_button.add_css_class("suggested-action")
        self.receive_button.connect("clicked", self.on_receive_clicked)
        container.append(self.receive_button)
        
        footer_note = Gtk.Label(label="Once you enter a valid ticket, the files will be ready to download.")
        footer_note.add_css_class("dim-label")
        footer_note.set_margin_top(24)
        container.append(footer_note)
        
        page.append(container)
        return page

    def on_nav_row_selected(self, listbox, row):
        if row:
            self.selected_tab = row.id
            self.content_stack.set_visible_child_name(self.selected_tab)

    def update_view(self):
        # Sync navigation selection
        if self.selected_tab == "send":
            self.nav_list.select_row(self.send_row)
        else:
            self.nav_list.select_row(self.receive_row)

    def on_select_file_clicked(self, button):
        dialog = Gtk.FileChooserDialog(
            title="Select File",
            parent=self,
            action=Gtk.FileChooserAction.OPEN
        )
        dialog.add_button("Cancel", Gtk.ResponseType.CANCEL)
        dialog.add_button("Select", Gtk.ResponseType.OK)
        
        dialog.connect("response", self.on_file_chooser_response)
        dialog.show()

    def on_file_chooser_response(self, dialog, response):
        if response == Gtk.ResponseType.OK:
            self.selected_file_path = dialog.get_file().get_path()
            self.drop_label.set_label(os.path.basename(self.selected_file_path))
            self.send_button.set_sensitive(True)
        dialog.destroy()

    def on_send_clicked(self, button):
        if self.selected_file_path:
            self.send_button.set_sensitive(False)
            thread = threading.Thread(target=self.do_send)
            thread.start()

    def do_send(self):
        try:
            ticket = self.core.send(self.selected_file_path)
            GLib.idle_add(self.show_ticket, ticket)
        except Exception as e:
            print(f"Send error: {e}")
            GLib.idle_add(self.send_button.set_sensitive, True)

    def show_ticket(self, ticket):
        self.current_ticket = ticket
        self.ticket_entry.set_text(ticket)
        self.ticket_box.set_visible(True)
        self.generate_qr(ticket)

    def generate_qr(self, data):
        qr = qrcode.QRCode(version=1, box_size=10, border=5)
        qr.add_data(data)
        qr.make(fit=True)
        img = qr.make_image(fill_color="black", back_color="white")
        
        buf = BytesIO()
        img.save(buf, format="PNG")
        image_data = buf.getvalue()
        
        loader = GdkPixbuf.PixbufLoader.new_with_type("png")
        loader.write(image_data)
        loader.close()
        pixbuf = loader.get_pixbuf()
        self.qr_image.set_from_pixbuf(pixbuf)

    def on_receive_clicked(self, button):
        ticket = self.receive_entry.get_text()
        if ticket:
            self.receive_button.set_sensitive(False)
            thread = threading.Thread(target=self.do_receive, args=(ticket,))
            thread.start()

    def do_receive(self, ticket):
        try:
            # Download to ~/Downloads
            dest_dir = os.path.expanduser("~/Downloads")
            self.core.receive(ticket, dest_dir)
            print("Download complete")
        except Exception as e:
            print(f"Receive error: {e}")
        finally:
            GLib.idle_add(self.receive_button.set_sensitive, True)

    def on_paste_clicked(self, button):
        clipboard = self.get_display().get_clipboard()
        clipboard.read_text_async(None, self.on_clipboard_read)

    def on_clipboard_read(self, clipboard, result):
        text = clipboard.read_text_finish(result)
        if text:
            self.receive_entry.set_text(text)

    def on_mode_toggled(self, switch, pspec):
        self.is_folder_mode = switch.get_active()
        if self.is_folder_mode:
            print("Switching to Folder mode")
        else:
            print("Switching to File mode")
