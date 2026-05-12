import gi
import os
import threading
import qrcode
from io import BytesIO
from PIL import Image

gi.require_version('Gtk', '4.0')
gi.require_version('Adw', '1')
from gi.repository import Gtk, Adw, Gdk, Gio, GLib

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
        
        # Main Layout: Navigation Split View
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
        css_provider.load_from_path(css_path)
        Gtk.StyleContext.add_provider_for_display(
            Gdk.Display.get_default(),
            css_provider,
            Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION
        )

    def create_sidebar(self):
        sidebar_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        sidebar_box.add_css_class("sidebar")
        sidebar_box.set_size_request(220, -1)

        # Logo Section
        logo_container = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12)
        logo_container.set_margin_start(16)
        logo_container.set_margin_end(16)
        logo_container.set_margin_top(20)
        logo_container.set_margin_bottom(32)

        logo_circle = Gtk.Box()
        logo_circle.add_css_class("sidebar-logo-circle")
        logo_label = Gtk.Label(label="S")
        logo_label.set_halign(Gtk.Align.CENTER)
        logo_label.set_valign(Gtk.Align.CENTER)
        logo_label.set_hexpand(True)
        logo_label.set_vexpand(True)
        logo_circle.append(logo_label)
        logo_container.append(logo_circle)

        text_container = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=4)
        brand_label = Gtk.Label(label="SEYFR")
        brand_label.add_css_class("sidebar-logo-text")
        brand_label.set_halign(Gtk.Align.START)
        text_container.append(brand_label)

        subtitle_label = Gtk.Label(label="Send Your Files Right")
        subtitle_label.add_css_class("sidebar-logo-subtitle")
        subtitle_label.set_halign(Gtk.Align.START)
        text_container.append(subtitle_label)
        
        logo_container.append(text_container)
        sidebar_box.append(logo_container)

        # Navigation Section
        nav_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=4)
        nav_box.set_margin_start(12)
        nav_box.set_margin_end(12)

        self.send_btn = Gtk.ToggleButton(label="Send")
        self.send_btn.add_css_class("nav-button")
        self.send_btn.set_active(True)
        self.send_btn.connect("clicked", self.on_tab_changed, "send")
        nav_box.append(self.send_btn)

        self.receive_btn = Gtk.ToggleButton(label="Receive")
        self.receive_btn.add_css_class("nav-button")
        self.receive_btn.set_group(self.send_btn)
        self.receive_btn.connect("clicked", self.on_tab_changed, "receive")
        nav_box.append(self.receive_btn)

        sidebar_box.append(nav_box)

        # Spacer
        spacer = Gtk.Box()
        spacer.set_vexpand(True)
        sidebar_box.append(spacer)

        # Status Section
        status_container = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=6)
        status_container.set_margin_start(16)
        status_container.set_margin_bottom(20)

        online_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=6)
        dot = Gtk.Box()
        dot.add_css_class("status-dot")
        online_box.append(dot)
        online_label = Gtk.Label(label="Online")
        online_label.add_css_class("caption")
        online_label.set_font_options(Gdk.FontOptions()) # For bold
        online_box.append(online_label)
        status_container.append(online_box)

        self.status_desc = Gtk.Label(label="Ready to send files")
        self.status_desc.add_css_class("caption")
        self.status_desc.set_halign(Gtk.Align.START)
        self.status_desc.set_opacity(0.6)
        status_container.append(self.status_desc)

        sidebar_box.append(status_container)

        sidebar_page = Adw.NavigationPage.new(sidebar_box, "Sidebar")
        self.split_view.set_sidebar(sidebar_page)

    def create_content(self):
        self.stack = Gtk.Stack()
        self.stack.set_transition_type(Gtk.StackTransitionType.CROSSFADE)

        # Send Page
        self.create_send_page()
        
        # Receive Page
        self.create_receive_page()

        content_page = Adw.NavigationPage.new(self.stack, "Content")
        self.split_view.set_content(content_page)

    def create_send_page(self):
        send_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=32)
        send_box.set_margin_top(40)
        send_box.set_margin_bottom(40)
        send_box.set_margin_start(40)
        send_box.set_margin_end(40)
        send_box.set_halign(Gtk.Align.CENTER)
        send_box.set_size_request(600, -1)

        # Header
        header_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=8)
        title = Gtk.Label(label="Send")
        title.add_css_class("detail-header-title")
        header_box.append(title)
        subtitle = Gtk.Label(label="Send your files to any device")
        subtitle.add_css_class("detail-header-subtitle")
        header_box.append(subtitle)
        send_box.append(header_box)

        # Drop Area Container
        self.drop_stack = Gtk.Stack()
        
        # Idle State (Drop Zone)
        self.idle_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=16)
        
        rings_overlay = Gtk.Overlay()
        rings_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)
        rings_box.set_size_request(280, 280)
        rings_box.set_halign(Gtk.Align.CENTER)
        
        # Add rings (mocked with layered circles for now)
        for i in range(8):
            ring = Gtk.Box()
            ring.add_css_class("concentric-ring")
            size = 80 + i * 22
            ring.set_size_request(size, size)
            # This is complex in GTK without custom drawing, but we'll use a DrawingArea later if needed
        
        # Simplified icon for now
        self.file_icon = Gtk.Image.new_from_icon_name("document-send-symbolic")
        self.file_icon.set_pixel_size(48)
        rings_overlay.set_child(self.file_icon)
        
        self.idle_box.append(rings_overlay)
        
        text_vbox = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=4)
        drag_label = Gtk.Label(label="Drag & drop files here")
        drag_label.set_opacity(0.6)
        text_vbox.append(drag_label)
        
        browse_btn = Gtk.Button(label="or click to browse")
        browse_btn.add_css_class("flat")
        browse_btn.connect("clicked", self.on_browse_clicked)
        text_vbox.append(browse_btn)
        self.idle_box.append(text_vbox)
        
        # Mode Toggle
        mode_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=12)
        mode_box.set_halign(Gtk.Align.CENTER)
        self.mode_label = Gtk.Label(label="File mode")
        mode_box.append(self.mode_label)
        self.mode_switch = Gtk.Switch()
        self.mode_switch.connect("notify::active", self.on_mode_toggled)
        mode_box.append(self.mode_switch)
        self.idle_box.append(mode_box)
        
        self.drop_stack.add_named(self.idle_box, "idle")
        
        # Active State (Ticket Card)
        self.ticket_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=24)
        
        # File Info Card
        file_card = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=16)
        file_card.add_css_class("card")
        file_icon_large = Gtk.Image.new_from_icon_name("document-open-symbolic")
        file_icon_large.set_pixel_size(32)
        file_card.append(file_icon_large)
        file_info = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=4)
        self.filename_label = Gtk.Label(label="filename.zip")
        self.filename_label.set_halign(Gtk.Align.START)
        file_info.append(self.filename_label)
        self.file_status_label = Gtk.Label(label="Ready to share")
        self.file_status_label.set_halign(Gtk.Align.START)
        self.file_status_label.set_opacity(0.6)
        file_info.append(self.file_status_label)
        file_card.append(file_info)
        self.ticket_box.append(file_card)
        
        # Ticket Card
        ticket_card = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=18)
        ticket_card.add_css_class("card")
        
        ticket_header = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL)
        ticket_title = Gtk.Label(label="Transfer Ticket")
        ticket_title.add_css_class("title-4")
        ticket_header.append(ticket_title)
        ticket_card.append(ticket_header)
        
        # QR Code Placeholder
        self.qr_image = Gtk.Image()
        self.qr_image.set_size_request(200, 200)
        ticket_card.append(self.qr_image)
        
        self.ticket_label = Gtk.Label()
        self.ticket_label.add_css_class("ticket-text")
        self.ticket_label.set_wrap(True)
        self.ticket_label.set_max_width_chars(60)
        ticket_card.append(self.ticket_label)
        
        btn_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=12)
        copy_btn = Gtk.Button(label="Copy")
        copy_btn.add_css_class("action-button")
        copy_btn.set_hexpand(True)
        copy_btn.connect("clicked", self.on_copy_ticket)
        btn_box.append(copy_btn)
        share_btn = Gtk.Button(label="Share")
        share_btn.add_css_class("action-button")
        share_btn.set_hexpand(True)
        btn_box.append(share_btn)
        ticket_card.append(btn_box)
        
        self.ticket_box.append(ticket_card)
        
        self.drop_stack.add_named(self.ticket_box, "ticket")
        send_box.append(self.drop_stack)

        self.stack.add_named(send_box, "send")

    def create_receive_page(self):
        receive_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=32)
        receive_box.set_margin_top(40)
        receive_box.set_margin_start(40)
        receive_box.set_margin_end(40)
        receive_box.set_halign(Gtk.Align.CENTER)
        receive_box.set_size_request(600, -1)

        # Header
        header_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=8)
        title = Gtk.Label(label="Receive")
        title.add_css_class("detail-header-title")
        header_box.append(title)
        subtitle = Gtk.Label(label="Receive files from any device")
        subtitle.add_css_class("detail-header-subtitle")
        header_box.append(subtitle)
        receive_box.append(header_box)

        # Ticket Input Card
        input_card = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=18)
        input_card.add_css_class("card")
        
        input_header = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL)
        input_title = Gtk.Label(label="Enter ticket")
        input_title.add_css_class("title-4")
        input_header.append(input_title)
        
        btn_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        btn_box.set_halign(Gtk.Align.END)
        paste_btn = Gtk.Button(label="Paste")
        paste_btn.add_css_class("flat")
        paste_btn.connect("clicked", self.on_paste_ticket)
        btn_box.append(paste_btn)
        clear_btn = Gtk.Button(label="Clear")
        clear_btn.add_css_class("flat")
        clear_btn.connect("clicked", self.on_clear_ticket)
        btn_box.append(clear_btn)
        input_header.append(btn_box)
        input_card.append(input_header)
        
        self.ticket_entry = Gtk.TextView()
        self.ticket_entry.set_size_request(-1, 80)
        self.ticket_entry.set_wrap_mode(Gtk.WrapMode.WORD_CHAR)
        input_card.append(self.ticket_entry)
        receive_box.append(input_card)

        # Save Location Card
        save_card = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=18)
        save_card.add_css_class("card")
        save_header = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL)
        save_title = Gtk.Label(label="Save Location")
        save_header.append(save_title)
        change_btn = Gtk.Button(label="Change")
        change_btn.set_halign(Gtk.Align.END)
        change_btn.set_hexpand(True)
        save_header.append(change_btn)
        save_card.append(save_header)
        
        path_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=12)
        path_box.append(Gtk.Image.new_from_icon_name("folder-symbolic"))
        path_info = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)
        path_name = Gtk.Label(label="Downloads")
        path_name.set_halign(Gtk.Align.START)
        path_info.append(path_name)
        self.path_label = Gtk.Label(label=os.path.expanduser("~/Downloads"))
        self.path_label.set_halign(Gtk.Align.START)
        self.path_label.set_opacity(0.6)
        path_info.append(self.path_label)
        path_box.append(path_info)
        save_card.append(path_box)
        receive_box.append(save_card)

        # Action Button
        self.receive_action_btn = Gtk.Button(label="Receive File")
        self.receive_action_btn.add_css_class("suggested-action")
        self.receive_action_btn.add_css_class("action-button")
        self.receive_action_btn.connect("clicked", self.on_receive_clicked)
        receive_box.append(self.receive_action_btn)

        self.stack.add_named(receive_box, "receive")

    def on_tab_changed(self, button, tab_name):
        if button.get_active():
            self.selected_tab = tab_name
            self.update_view()

    def update_view(self):
        self.stack.set_visible_child_name(self.selected_tab)
        if self.selected_tab == "send":
            self.status_desc.set_text("Ready to send files")
        else:
            self.status_desc.set_text("Ready to receive files")

    def on_mode_toggled(self, switch, gparam):
        self.is_folder_mode = switch.get_active()
        icon = "folder-symbolic" if self.is_folder_mode else "document-send-symbolic"
        self.file_icon.set_from_icon_name(icon)

    def on_browse_clicked(self, button):
        dialog = Gtk.FileDialog()
        if self.is_folder_mode:
            dialog.select_folder(self, None, self.on_file_selected)
        else:
            dialog.open(self, None, self.on_file_selected)

    def on_file_selected(self, dialog, result):
        try:
            if self.is_folder_mode:
                file = dialog.select_folder_finish(result)
            else:
                file = dialog.open_finish(result)
            
            if file:
                self.selected_file_path = file.get_path()
                self.filename_label.set_text(os.path.basename(self.selected_file_path))
                self.start_send()
        except Exception as e:
            print(f"Error selecting file: {e}")

    def start_send(self):
        self.file_status_label.set_text("Generating ticket...")
        self.drop_stack.set_visible_child_name("ticket")
        
        def run_send():
            try:
                ticket = self.core.send(self.selected_file_path)
                GLib.idle_add(self.on_send_complete, ticket)
            except Exception as e:
                GLib.idle_add(self.on_send_error, str(e))

        threading.Thread(target=run_send, daemon=True).start()

    def on_send_complete(self, ticket):
        self.current_ticket = ticket
        self.ticket_label.set_text(ticket)
        self.file_status_label.set_text("Completed")
        self.generate_qr(ticket)

    def on_send_error(self, error):
        self.file_status_label.set_text(f"Error: {error}")

    def generate_qr(self, data):
        qr = qrcode.QRCode(version=1, box_size=10, border=4)
        qr.add_data(data)
        qr.make(fit=True)
        img = qr.make_image(fill_color="black", back_color="white")
        
        # Convert PIL to GdkPixbuf
        buffer = BytesIO()
        img.save(buffer, format="PNG")
        bytes_data = buffer.getvalue()
        
        loader = Gdk.PixbufLoader.new_with_type("png")
        loader.write(bytes_data)
        loader.close()
        pixbuf = loader.get_pixbuf()
        
        # Convert Pixbuf to Texture for Gtk4
        texture = Gdk.Texture.new_for_pixbuf(pixbuf)
        self.qr_image.set_from_paintable(texture)

    def on_copy_ticket(self, button):
        if self.current_ticket:
            clipboard = self.get_display().get_clipboard()
            clipboard.set(self.current_ticket)

    def on_paste_ticket(self, button):
        clipboard = self.get_display().get_clipboard()
        clipboard.read_text_async(None, self.on_paste_done)

    def on_paste_done(self, clipboard, result):
        text = clipboard.read_text_finish(result)
        if text:
            buffer = self.ticket_entry.get_buffer()
            buffer.set_text(text)

    def on_clear_ticket(self, button):
        self.ticket_entry.get_buffer().set_text("")

    def on_receive_clicked(self, button):
        buffer = self.ticket_entry.get_buffer()
        ticket = buffer.get_text(buffer.get_start_iter(), buffer.get_end_iter(), False)
        dest_dir = self.path_label.get_text()
        
        self.receive_action_btn.set_sensitive(False)
        
        def run_receive():
            try:
                self.core.receive(ticket, dest_dir)
                GLib.idle_add(self.on_receive_complete)
            except Exception as e:
                GLib.idle_add(self.on_receive_error, str(e))

        threading.Thread(target=run_receive, daemon=True).start()

    def on_receive_complete(self):
        self.receive_action_btn.set_sensitive(True)
        # Show success message/pill

    def on_receive_error(self, error):
        self.receive_action_btn.set_sensitive(True)
        print(f"Receive error: {error}")
