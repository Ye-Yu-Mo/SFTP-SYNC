<div align="center">

# SFTP-SYNC

**一个现代化的跨平台 GUI 应用，可将您的本地文件自动同步到 SFTP 服务器。**

</div>

<p align="center">
  <a href="README.md">English</a> | 
  <a href="README.zh-CN.md">简体中文</a> | 
  <a href="README.ja-JP.md">日本語</a> | 
  <a href="README.ko-KR.md">한국어</a>
</p>

<p align="center">
  <img src="https://img.shields.io/github/v/release/Ye-Yu-Mo/SFTP-SYNC?style=for-the-badge" alt="发布版本"/>
  <img src="https://img.shields.io/github/actions/workflow/status/Ye-Yu-Mo/SFTP-SYNC/release.yml?style=for-the-badge" alt="构建状态"/>
  <img src="https://img.shields.io/github/license/Ye-Yu-Mo/SFTP-SYNC?style=for-the-badge" alt="许可证"/>
</p>

## 功能特性

- **实时同步**: 自动监控本地目录中的文件变更，并立即上传。
- **现代化 GUI**: 基于 [GPUI](https://gpui.dev/) 构建的快速、直观且由 GPU 加速的图形界面。
- **安全凭证存储**: 密码和 SSH 密钥的口令被安全地存储在您操作系统的原生钥匙串中。
- **灵活的认证方式**: 同时支持密码和 SSH 私钥两种认证方法。
- **多目标管理**: 在一个地方配置和管理多个同步目标。
- **带宽控制**: 可以选择限制上传带宽，以节省网络资源。
- **安全第一**: 在执行删除远程文件等破坏性操作前，会进行确认提示。
- **跨平台**: 可在 macOS、Windows 和 Linux 上运行。

## 安装

您可以从 [**GitHub Releases**](https://github.com/Ye-Yu-Mo/SFTP-SYNC/releases) 页面下载适用于您操作系统的最新版本。

或者，如果您已安装 Rust 环境，也可以从源代码构建：

```bash
git clone https://github.com/Ye-Yu-Mo/SFTP-SYNC.git
cd sftp-sync
cargo build --release
```

可执行文件将位于 `target/release` 目录中。

## 如何使用

1.  **启动应用**: 运行 SFTP-SYNC。
2.  **添加同步目标**:
    - 点击“添加新目标”。
    - 填写您的 SFTP 服务器信息：
      - **目标名称**: 为此连接设置一个易于识别的名称（例如，“我的 Web 服务器”）。
      - **主机**: 服务器地址（例如，`sftp.example.com`）。
      - **用户名**: 您的 SFTP 用户名。
      - **认证方式**: 选择“密码”或“SSH 密钥”。应用会将您的凭证安全地保存在系统钥匙串中。
      - **本地路径**: 您希望同步的本地目录。
      - **远程路径**: 服务器上与之对应的目录。
3.  **连接**: 点击您刚刚创建的目标旁边的“连接”按钮。
4.  **开始同步**: 连接成功后，应用将开始监控您的本地路径。任何新建、修改或删除的文件都将自动同步到远程服务器。

## 配置

应用的配置完全通过图形界面进行管理。所有的设置和同步目标都保存在位于系统标准配置目录下的 `config.json` 文件中。

**全局设置 (可在设置面板中找到):**

- **界面语言**: 选择您偏好的语言。
- **启动时自动连接**: 应用启动时自动连接到所有活动的目标。
- **监控本地变更**: 开启或关闭实时文件监控功能。
- **破坏性操作前确认**: 启用或禁用删除文件前的安全提示。
- **限制带宽**: 设置一个最大上传速度（单位：Mbps）。

## 贡献

欢迎各种形式的贡献！请随时提交 Issue 或 Pull Request。

## 许可证

本项目基于 [MIT 许可证](LICENSE) 授权。
