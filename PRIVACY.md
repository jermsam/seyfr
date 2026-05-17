# SEYFR Privacy Policy

**Effective Date:** May 17, 2026  
**Provider:** JITPOMI LLC ("we", "us", or "our")

SEYFR is a decentralized, secure peer-to-peer file sharing application built on top of the Iroh protocol. This Privacy Policy explains our data practices when you install and use the SEYFR mobile application.

## 1. Data Collection & Storage
SEYFR operates on a direct peer-to-peer (P2P) architecture:
* **Zero Cloud Storage:** We do not host, store, intercept, or mirror any files, photos, videos, or documents you transfer through SEYFR. All data transfers occur directly between the sender and receiver over encrypted QUIC connections.
* **Local Storage:** Any files received or transferred are stored strictly locally inside your device's protected application sandbox or user-designated media directories.
* **Account Data:** SEYFR does not require user registration, logins, passwords, or phone numbers to operate.

## 2. Network Connectivity & Peer Discovery
To establish direct P2P connections, SEYFR utilizes encrypted ticket strings (QR codes) containing ephemeral public keys and direct IP endpoints:
* **Relay Nodes:** If a direct NAT-traversal connection cannot be established between peers, SEYFR may route encrypted packets through public Iroh relay servers (DERP nodes) strictly for connection facilitation. Relay nodes cannot decrypt payload data.

## 3. Camera & Media Permissions
* **Camera Access:** SEYFR requests camera permissions solely for the purpose of scanning QR code tickets to initiate peer connection pairing. Camera streams are processed locally on device and never transmitted over the network.
* **Storage Access:** Storage permissions are required strictly to read selected files for transmission and save incoming files to disk upon user confirmation.

## 4. Third-Party Services
SEYFR includes optional public donation/monetization instructions for supporting development (via Mercury Bank and Venmo). We do not collect banking details, credit card numbers, or transactional analytics from your donations.

## 5. Contact Us
If you have any questions or concerns regarding this Privacy Policy or data privacy in SEYFR, please contact us at:
**Email:** dev@jitpomi.com
