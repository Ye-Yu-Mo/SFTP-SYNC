<div align="center">

# SFTP-SYNC

**로컬 파일을 SFTP 서버에 자동으로 동기화하는 최신 크로스플랫폼 GUI 애플리케이션입니다.**

</div>

<p align="center">
  <a href="README.md">English</a> | 
  <a href="README.zh-CN.md">简体中文</a> | 
  <a href="README.ja-JP.md">日本語</a> | 
  <a href="README.ko-KR.md">한국어</a>
</p>

<p align="center">
  <img src="https://img.shields.io/github/v/release/your-username/sftp-sync?style=for-the-badge" alt="릴리스 버전"/>
  <img src="https://img.shields.io/github/actions/workflow/status/your-username/sftp-sync/release.yml?style=for-the-badge" alt="빌드 상태"/>
  <img src="https://img.shields.io/github/license/your-username/sftp-sync?style=for-the-badge" alt="라이선스"/>
</p>

## 주요 기능

- **실시간 동기화**: 로컬 디렉토리의 변경 사항을 자동으로 감지하여 즉시 업로드합니다.
- **최신 GUI**: [GPUI](https://gpui.dev/)로 제작된 빠르고 직관적이며 GPU 가속을 지원하는 인터페이스.
- **안전한 자격 증명 저장**: 비밀번호와 SSH 키 암호는 운영 체제의 네이티브 키체인에 안전하게 저장됩니다.
- **유연한 인증 방식**: 비밀번호 및 SSH 개인 키 인증을 모두 지원합니다.
- **다중 대상 관리**: 여러 동기화 대상을 한 곳에서 설정하고 관리할 수 있습니다.
- **대역폭 제어**: 네트워크 리소스를 절약하기 위해 업로드 대역폭을 제한하는 옵션이 있습니다.
- **안전 우선**: 원격 파일 삭제와 같은 파괴적인 작업을 실행하기 전에 확인 프롬프트를 제공합니다.
- **크로스플랫폼**: macOS, Windows, Linux에서 실행됩니다.

## 설치

[**GitHub Releases**](https://github.com/your-username/sftp-sync/releases) 페이지에서 사용 중인 운영 체제에 맞는 최신 릴리스를 다운로드할 수 있습니다.

또는 Rust가 설치되어 있다면 소스에서 직접 빌드할 수도 있습니다:

```bash
git clone https://github.com/your-username/sftp-sync.git
cd sftp-sync
cargo build --release
```

실행 파일은 `target/release` 디렉토리에 생성됩니다.

## 사용 방법

1.  **애플리케이션 실행**: SFTP-SYNC를 시작합니다.
2.  **동기화 대상 추가**:
    - "Add New Target"을 클릭합니다.
    - SFTP 서버 정보를 입력합니다:
      - **Target Name**: 연결을 식별하기 쉬운 이름 (예: "My Web Server").
      - **Host**: 서버 주소 (예: `sftp.example.com`).
      - **Username**: SFTP 사용자 이름.
      - **Authentication**: "Password" 또는 "SSH Key" 중에서 선택합니다. 앱은 자격 증명을 OS 키체인에 안전하게 저장합니다.
      - **Local Path**: 동기화할 로컬 디렉토리.
      - **Remote Path**: 동기화될 서버의 해당 디렉토리.
3.  **연결**: 방금 생성한 대상의 "Connect" 버튼을 클릭합니다.
4.  **동기화 시작**: 연결되면 애플리케이션이 로컬 경로를 감시하기 시작합니다. 새로 생성되거나, 변경되거나, 삭제된 모든 파일은 자동으로 원격 서버에 미러링됩니다.

## 설정

애플리케이션 설정은 GUI를 통해 직접 관리됩니다. 모든 설정 및 동기화 대상은 시스템의 표준 설정 디렉토리에 있는 `config.json` 파일에 저장됩니다.

**전역 설정 (설정 패널에서 가능):**

- **UI 언어**: 선호하는 언어를 선택합니다.
- **시작 시 자동 연결**: 앱 시작 시 모든 활성 대상에 자동으로 연결합니다.
- **로컬 변경 사항 감시**: 실시간 파일 감시 기능을 켜거나 끕니다.
- **파괴적인 작업 전 확인**: 파일 삭제 전 안전 확인 프롬프트를 활성화/비활성화합니다.
- **대역폭 제한**: 최대 업로드 속도를 Mbps 단위로 설정합니다.

## 기여

기여를 환영합니다! 언제든지 이슈를 제기하거나 풀 리퀘스트를 제출해 주세요.

## 라이선스

이 프로젝트는 [MIT 라이선스](LICENSE)에 따라 라이선스가 부여됩니다.
