import os
from seyfr_core import Core, SeyfrError, ProgressSink

class CoreWrapper:
    """Wrapper for Rust Core bindings with proper API mapping"""
    
    def __init__(self, data_dir=None):
        if data_dir is None:
            # Default to ~/.local/share/seyfr on Linux
            data_dir = os.path.expanduser("~/.local/share/seyfr")
        
        if not os.path.exists(data_dir):
            os.makedirs(data_dir, exist_ok=True)
            
        self.core = Core(data_dir)
    
    def send(self, path, progress_sink=None):
        """Send a file or folder and return the ticket"""
        try:
            return self.core.send(path, progress_sink)
        except Exception as e:
            raise e

    def receive(self, ticket, dest_dir, progress_sink=None):
        """Receive a file or folder from a ticket"""
        try:
            self.core.receive(ticket, dest_dir, progress_sink)
        except Exception as e:
            raise e

    def node_id(self):
        """Get the local node ID"""
        return self.core.node_id()
