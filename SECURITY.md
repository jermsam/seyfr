# Security Policy

Filesystems are the "Wild West" of application security. Without these guardrails, a multiplatform app—which has to juggle different file rules for Windows, macOS, and Linux—could accidentally give an attacker the keys to the entire operating system.

Here is why each of those features is a non-negotiable part of a secure app's DNA:

## 1. Path Traversal Protection (path_jail)

### The Threat

An attacker tries to "escape" the folder you've given them access to by using sequences like `../` (dot-dot-slash). If your app asks for a profile picture and the user provides `../../../../etc/shadow`, a vulnerable app might actually serve up the system's encrypted password file.

### The Solution

A `path_jail` (or chroot-like mechanism) forces the application to treat a specific directory as the "root." Even if a malicious path contains 50 "go up" commands, the app physically cannot see anything above its assigned sandbox.

## 2. Why Seyfr Has NO Hard File Size Limits

### The P2P Difference

Unlike server-based file sharing (Dropbox, Google Drive), **Seyfr is peer-to-peer**. This fundamentally changes the security model:

| Scenario | Server-Based App | P2P App (Seyfr) |
|----------|---------------|----------------|
| Who's disk fills? | **Company's servers** | **Your own device** |
| Who pays for storage? | **Company** | **You** |
| Unsolicited attack? | **Yes** - anyone can upload | **No** - you must choose to receive |
| Shared resource? | **Yes** - affects other users | **No** - only affects you |

### Why Size Limits Make Sense for Dropbox But NOT for Seyfr

**Dropbox/Google Drive need limits because:**
- You're filling **THEIR** servers with **YOUR** data
- Malicious users can upload 500 GB to abuse free tiers
- Storage costs are borne by the company
- One user's abuse affects all other users (shared infrastructure)

**Seyfr doesn't need limits because:**
- You're transferring between **YOUR** devices
- You can't "DoS yourself" by choice
- No shared infrastructure to abuse
- iroh-blobs streams data efficiently (no loading entire files in memory)
- The recipient must **actively choose** to accept the transfer

### The Real P2P Threat: Zip Slip & Path Traversal

What P2P apps actually need to worry about isn't file **size** - it's file **content** and **paths**:
- A malicious sender could include `../../etc/shadow` as a filename
- A symlink could point to `C:\Windows\System32`
- These are the attacks that matter for P2P

**Seyfr protects against these with:**
- `path_jail` for path containment (Section 1)
- Symlink skipping (Section 4)
- Destination validation (Section 3)

## 3. Destination Validation

### The Threat

Even without "traversal" tricks, an app might be tricked into writing files to sensitive locations it technically has permission to access, like a startup folder or a shared library directory.

### The Solution

This ensures the app explicitly checks, "Is this specific folder on my 'Allowed' list?" before it touches the disk. It's the difference between a bouncer checking your ID and a bouncer checking if you're even on the guest list.

## 4. Symlink Safety

### The Threat

Symbolic links (symlinks) are "shortcuts." An attacker can include a symlink in a transfer that looks like a harmless text file but actually points to `/etc/passwd` or `C:\Windows\System32`. If the app follows that link during a write operation, it might overwrite a critical system file.

### The Solution

By skipping symlinks by default, the app refuses to follow these "redirects," ensuring it only interacts with the actual data provided, not pointers to the rest of the machine.

---

## Summary of Protections

| Feature | Primary Goal | Prevents... |
| --- | --- | --- |
| Path Jail | Containment | Accessing OS system files |
| Destination Validation | Integrity | Overwriting unintended directories |
| Symlink Safety | Escape Prevention | "Zip Slip" and shortcut-based exploits |

---

## Reporting a Vulnerability

If you discover a security vulnerability in Seyfr, please report it responsibly. We take security seriously and will work to address issues promptly.

[Add your preferred reporting mechanism here - e.g., email, GitHub Security Advisories, etc.]
