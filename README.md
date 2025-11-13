<div align="center">

# SFTP-SYNC

**A modern, cross-platform GUI application to automatically sync your local files to an SFTP server.**

</div>

<p align="center">
  <a href="README.md">English</a> | 
  <a href="README.zh-CN.md">简体中文</a> | 
  <a href="README.ja-JP.md">日本語</a> | 
  <a href="README.ko-KR.md">한국어</a>
</p>

<p align="center">
  <img src="https://img.shields.io/github/v/release/Ye-Yu-Mo/SFTP-SYNC?style=for-the-badge" alt="Release Version"/>
  <img src="https://img.shields.io/github/actions/workflow/status/Ye-Yu-Mo/SFTP-SYNC/release.yml?style=for-the-badge" alt="Build Status"/>
  <img src="https://img.shields.io/github/license/Ye-Yu-Mo/SFTP-SYNC?style=for-the-badge" alt="License"/>
</p>

## Features

- **Real-Time Sync**: Automatically watches for changes in your local directory and uploads them instantly.
- **Modern GUI**: A fast, intuitive, and GPU-accelerated interface built with [GPUI](https://gpui.dev/).
- **Secure Credential Storage**: Passwords and SSH key passphrases are securely stored in your operating system's native keychain.
- **Flexible Authentication**: Supports both password and SSH private key authentication.
- **Multiple Targets**: Configure and manage multiple synchronization targets in one place.
- **Bandwidth Control**: Option to limit the upload bandwidth to save network resources.
- **Safety First**: Provides confirmation prompts before executing destructive operations like deleting remote files.
- **Cross-Platform**: Runs on macOS, Windows, and Linux.

## Installation

You can download the latest release for your operating system from the [**GitHub Releases**](https://github.com/Ye-Yu-Mo/SFTP-SYNC/releases) page.

Alternatively, if you have Rust installed, you can build it from source:

```bash
git clone https://github.com/Ye-Yu-Mo/SFTP-SYNC.git
cd sftp-sync
cargo build --release
```

The executable will be located in the `target/release` directory.

## How to Use

1.  **Launch the Application**: Start SFTP-SYNC.
2.  **Add a Sync Target**:
    - Click on "Add New Target".
    - Fill in the details for your SFTP server:
      - **Target Name**: A friendly name for this connection (e.g., "My Web Server").
      - **Host**: The server's address (e.g., `sftp.example.com`).
      - **Username**: Your SFTP username.
      - **Authentication**: Choose between "Password" or "SSH Key". The app will securely save your credentials in the OS keychain.
      - **Local Path**: The local directory you want to sync from.
      - **Remote Path**: The corresponding directory on the server you want to sync to.
3.  **Connect**: Click the "Connect" button for the target you just created.
4.  **Start Syncing**: Once connected, the application will begin watching your local path. Any new files, changes, or deletions will be automatically mirrored to the remote server.

## Configuration

The application's configuration is managed directly through the GUI. All settings and sync targets are saved to a `config.json` file located in your system's standard config directory.

**Global Settings (available in the Settings panel):**

- **UI Language**: Choose your preferred language.
- **Auto-Connect on Startup**: Automatically connect to all active targets when the app starts.
- **Watch for Local Changes**: Toggle the real-time file watching feature.
- **Confirm Destructive Operations**: Enable/disable safety prompts before deleting files.
- **Limit Bandwidth**: Set a maximum upload speed in Mbps.

## Contributing

Contributions are welcome! Please feel free to open an issue or submit a pull request.

## License

This project is licensed under the [MIT License](LICENSE).