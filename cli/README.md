# seyfr-cli

Development CLI for testing P2P file transfers.

## Purpose

This binary is a minimal wrapper around `seyfr-core` to exercise the full send/receive flow from the command line. It is **not a platform target** — it exists to verify that the core library, iroh networking, and blob export all work correctly before wiring them into native iOS/Android/Web UIs.

## Build

From the workspace root:

```bash
cargo build -p seyfr-cli
```

Or run directly:

```bash
cargo run -p seyfr-cli -- --help
```

## Commands

### `send`

Share a file or folder and return a compact ticket string.

```bash
seyfr send /path/to/file.txt --listen
```

| Flag | Description |
|---|---|
| `--listen` | **Keep the node alive** after generating the ticket so the receiver can download. |
| `-d, --data-dir <DIR>` | Directory for the iroh node store (default: `.seyfr`). |

**Example output:**

```
Node ID: 5e0700ecdbcb396a20c5ca45c41671489764dcf38c391628efef76a49100586d
Sending: /tmp/photo.jpg
Generating ticket...

[  1/1  ] Starting: photo.jpg
[  1/1  ] Complete: photo.jpg

🎫 Ticket (share this with recipient):
blobacgwm2idvanznqhp64oif52hq5zic35renouiashh5c7zn6yo6bnqaabadakqaio2dfqgacbxfod37d5qtedag2ysf4nw6kzop3lpdy72ajwv2yb67v4cykhcq

🔊 Listening for connections... Press Ctrl+C to stop.
```

Copy the `blob...` ticket string to give to the receiver.

### `receive`

Download files from a ticket into a destination directory.

```bash
seyfr receive "<ticket>" -d /path/to/destination
```

| Flag | Description |
|---|---|
| `-d, --dest <DIR>` | Destination directory for received files (default: `.`). |
| `--data-dir <DIR>` | Directory for the iroh node store (default: `.seyfr`). |

**Example output:**

```
Node ID: debbf5fdd24a243dbb8a53291f6e3feb475b7d5a261b1bdbd51d79d3f371762c
Receiving to: /tmp/received

[  1/1  ] Starting: file
  file: 35 bytes
[  1/1  ] Complete: file

Transfer complete.
```

### `node-id`

Print the node ID for the configured data directory (useful for debugging network connectivity).

```bash
seyfr node-id
```

## End-to-End Test

Open two terminal windows.

**Terminal 1 — Sender:**

```bash
seyfr send /path/to/file.txt --listen
```

**Terminal 2 — Receiver:**

```bash
seyfr receive "<paste-ticket-here>" -d /tmp/dest
```

If the nodes are on the same machine, use separate `--data-dir` values to avoid store conflicts:

```bash
# Sender
seyfr send /path/to/file.txt --listen -d /tmp/sender-node

# Receiver
seyfr receive "<ticket>" -d /tmp/dest --data-dir /tmp/receiver-node
```

## Notes

- **Separate data directories:** Always use different `--data-dir` paths for sender and receiver when testing on the same machine, otherwise the iroh store will conflict.
- **Ctrl+C to stop:** The `--listen` flag blocks until interrupted. This is intentional — the sender node must stay online for the receiver to connect and download.
- **Progress reporting:** The CLI implements the `ProgressSink` trait from `seyfr-core` to print real-time progress to stdout.
- **Not for production:** This tool is for development and integration testing only. It does not handle backgrounding, daemonization, or platform-native UI integration.
