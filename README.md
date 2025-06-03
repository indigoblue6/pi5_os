# pi5_os
Raspberry Pi 5 Baremetal OS like UNIX

Raspberry Pi 5用のベアメタルOSプロジェクト。UNIXライクなシェル機能を持つミニマルなカーネル実装です。

## 概要

このプロジェクトは、Raspberry Pi 5上で動作するベアメタルOSの実装です。基本的なシェル機能、プロセス管理、ファイルシステム、タイマー、GPIO制御などの機能を提供します。

## 特徴

- **ベアメタル実装**: OSやライブラリに依存しない直接ハードウェア制御
- **UNIXライクシェル**: 基本的なコマンドラインインターフェース
- **マルチプロセス対応**: プロセス管理とスケジューリング機能
- **MMU制御**: メモリ管理ユニットによるメモリ保護
- **割り込み処理**: タイマーやUART割り込みのサポート
- **GPIO制御**: Raspberry Pi 5のGPIOピン制御
- **ファイルシステム**: 基本的なファイル操作機能

## 参考プロジェクト

このプロジェクトは以下のリポジトリを参考にして開発されました：

- **[tnishinaga/pi5_hack](https://github.com/tnishinaga/pi5_hack)** - Raspberry Pi 5のベアメタルプログラミングのサンプル集
  - Pi5の基本的なハードウェア制御方法
  - UART通信の実装（RP1チップ経由のPL011 UART）
  - MMUとページテーブルの設定
  - 割り込み処理の実装
  - OSカーネルの基本構造
  - システムタイマーの制御方法

特に以下の実装を参考にしています：
- `XX_pi5_os/` - OSカーネルの基本構造とシェル実装
- `XX_early_uart/` - 初期UART通信の実装
- `XX_systimer/` - システムタイマーの制御

## 機能

### システム機能
- **ブートローダー**: ARM64アセンブリによる起動シーケンス
- **UART通信**: RP1チップ経由のPL011 UART（Ubuntu kernel準拠）
- **プロセス管理**: UNIX風のプロセス管理（PID、親子関係）
- **システムタイマー**: BCM2712システムタイマー
- **MMU制御**: メモリ管理ユニットによるメモリ保護
- **割り込み処理**: タイマーやUART割り込みのサポート
- **GPIO制御**: Raspberry Pi 5のGPIOピン制御
- **ファイルシステム**: 基本的なファイル操作機能

## ディレクトリ構成

```
minimal_pi5_os/
├── src/
│   ├── main.rs          # エントリーポイントとカーネル初期化
│   ├── startup.s        # アセンブリ起動コード
│   ├── uart.rs          # UART通信モジュール
│   ├── mmu.rs           # メモリ管理ユニット
│   ├── process.rs       # プロセス管理
│   ├── shell.rs         # シェルインターフェース
│   ├── interrupt.rs     # 割り込み処理
│   ├── timer.rs         # タイマー制御
│   ├── gpio.rs          # GPIO制御
│   └── filesystem.rs    # ファイルシステム
├── Cargo.toml           # Rustプロジェクト設定
├── Makefile             # ビルドスクリプト
├── ldscript.lds         # リンカースクリプト
├── config.txt           # Pi5起動設定
└── README.md            # このファイル
```

## 必要な環境

### ハードウェア
- Raspberry Pi 5
- microSDカード（16GB以上推奨）
- UART通信用ケーブル（デバッグ用）

### ソフトウェア
- Rust（nightly版推奨）
- aarch64-unknown-none ターゲット
- aarch64-linux-gnu-objcopy（バイナリ生成用）
- QEMU（テスト用、オプション）

## インストールとセットアップ

### 1. Rust環境の準備

```bash
# Rustのインストール（未インストールの場合）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# nightly版の設定
rustup default nightly

# aarch64ターゲットの追加
rustup target add aarch64-unknown-none
```

### 2. クロスコンパイルツールの準備

```bash
# Debian/Ubuntu
sudo apt install gcc-aarch64-linux-gnu

# macOS (Homebrew)
brew install aarch64-elf-gcc
```

### 3. プロジェクトのビルド

```bash
# リポジトリのクローン
git clone <このリポジトリのURL>
cd minimal_pi5_os

# ビルド
make

# kernel8.imgが生成されることを確認
ls -la kernel8.img
```

## Raspberry Pi 5での実行

### 1. SDカードの準備

1. Raspberry Pi OS Liteをインストール
2. `/boot/firmware/`に以下のファイルをコピー：
   - `kernel8.img`（ビルドで生成されたファイル）
   - `config.txt`（このリポジトリのもの）

### 2. config.txtの設定

```ini
# Pi5用設定
arm_64bit=1
kernel=kernel8.img
disable_splash=1

# UART有効化（デバッグ用）
enable_uart=1
uart_2ndstage=1

# その他の設定
dtoverlay=disable-bt
```

### 3. 起動とシェル操作

1. SDカードをPi5に挿入して起動
2. UART接続でシェルにアクセス
3. 基本コマンドの実行例：

```bash
> help
> ls
> ps
> echo Hello, Pi5 OS!
> clear
```

## シェルコマンド

### 実装済みコマンド
```
- `help` - ヘルプ表示とコマンド一覧
- `ps` - プロセス一覧表示
- `uptime` - システム稼働時間
- `uname` - システム情報表示
- `echo` - 文字列出力
- `clear` - 画面クリア
- `history` - コマンド履歴表示
- `date` - 現在時刻表示
- `whoami` - ユーザー名表示
- `pwd` - 現在ディレクトリ表示
- `ls` - ディレクトリ一覧表示
- `cat` - ファイル内容表示
- `test` - システムテスト実行
- `reboot` - システム再起動
- `exit` - シェル終了

### シェル機能
- **コマンド履歴**: 最大10件のコマンド履歴
- **プロセス情報**: `/proc/`風の情報表示
- **システムテスト**: ハードウェア動作確認
- **インタラクティブ**: リアルタイム入力処理

## 開発とデバッグ

### QEMUでのテスト

```bash
# 基本的な動作確認（30秒でタイムアウト）
make qemu
```

### デバッグビルド

```bash
# デバッグ版のビルド
cargo build

# デバッグ情報付きバイナリの生成
aarch64-linux-gnu-objcopy -O binary target/aarch64-unknown-none/debug/minimal_pi5_os kernel8.img
```

## 実装されている機能

### システム機能
- プロセス作成と管理
- 基本的なスケジューリング
- メモリ保護（MMU使用）
- タイマー割り込み
- UART通信
- GPIO制御

### pi5_hackからの学習要素

このプロジェクトは`tnishinaga/pi5_hack`から以下の重要な実装技術を学習しています：

#### UART通信の実装
- RP1チップ経由のPL011 UART制御
- 115200 baud、8N1設定
- 割り込み駆動の入出力処理

#### メモリ管理
- ARM64のページテーブル設定
- MMUによるメモリ保護
- プロセス毎のメモリ空間分離

#### 起動シーケンス
- ARMアセンブリによるブートコード
- スタックポインタの初期化
- C/Rustコードへの制御移譲

#### システムタイマー
- BCM2712システムタイマーの制御
- 定期割り込みの設定
- タイムスライシングの実装
```

## アーキテクチャと技術仕様

### システムアーキテクチャ
- **CPU**: ARM Cortex-A76 (64-bit)
- **SoC**: BCM2712 (Raspberry Pi 5)
- **I/Oコントローラー**: RP1チップ
- **メモリ**: 最小1GB RAM必要
- **起動アドレス**: 0x200000

### メモリレイアウト
```
0x00000000 - 0x001FFFFF  ペリフェラル領域
0x00200000 - 0x002FFFFF  カーネルコード領域
0x00300000 - 0x004FFFFF  カーネルスタック
0x00500000 - 0x00FFFFFF  プロセス領域
0x01000000 - 0xFFFFFFFF  ユーザー空間
```

### ハードウェア対応
- **UART**: PL011（RP1チップ経由）
- **GPIO**: BCM2712 + RP1 GPIO制御
- **タイマー**: BCM2712システムタイマー
- **割り込み**: GIC-400割り込みコントローラー

### Ubuntu準拠実装
このOSは、Ubuntu 24.04 "Noble"のRaspberry Pi 5対応を参考に実装されています：
```
カーネル: linux-raspi 6.8系ベース
GPIO: pinctrl-rp1.cドライバー準拠  
UART: amba-pl011.cドライバー準拠
Timer: BCM2712システムタイマー
```

## 技術仕様

- **言語**: Rust + ARM64 Assembly
- **ターゲット**: aarch64-unknown-none
- **最小メモリ**: 1GB RAM
- **UART設定**: 115200 baud, 8N1
- **起動時間**: 約3秒
- **プロセススタック**: 2MB (プロセス毎)
- **プロセス空間**: 1MB単位で分離

## 貢献とサポート

### 実装済み
- ✅ ブートローダー
- ✅ UART初期化・通信
- ✅ 基本プロセス管理
- ✅ インタラクティブシェル
- ✅ システムタイマー
- ✅ 基本的なコマンド群

### 長期的なビジョン
- より完全なUNIXライクOS の実現
- 教育用プラットフォームとしての活用
- リアルタイムシステムの対応
- 組み込みアプリケーション向けの最適化

## ハードウェアテスト手順

### 必要なもの
- Raspberry Pi 5 (4GB以上推奨)
- MicroSDカード (8GB以上)
- USB-UART変換ケーブル
- ジャンパーワイヤー

### 接続方法
```
Pi5 GPIO    | UART変換
-----------+----------
GPIO 14 TX  | RX (白)
GPIO 15 RX  | TX (緑)  
GND        | GND (黒)
```

### SDカード準備
```bash
# FAT32でフォーマット
sudo mkfs.fat -F 32 /dev/sdX1

# 必要ファイルをコピー
sudo mount /dev/sdX1 /mnt
sudo cp kernel8.img /mnt/
sudo cp config.txt /mnt/
sudo umount /mnt
```

### 動作確認
```bash
# UART接続（Linux）
screen /dev/ttyUSB0 115200

# UART接続（macOS）  
screen /dev/cu.usbserial 115200

# UART接続（Windows）
putty -serial COM3 -sercfg 115200,8,n,1
```

## ライセンス

このプロジェクトはMITライセンスの下で公開されています。詳細は[LICENSE](LICENSE)ファイルを参照してください。

## 参考文献と謝辞

### 主要参考文献
- **[tnishinaga/pi5_hack](https://github.com/tnishinaga/pi5_hack)** - Pi5 ベアメタルプログラミングの基礎を提供
- [Raspberry Pi 5 公式ドキュメント](https://www.raspberrypi.com/documentation/computers/raspberry-pi.html)
- [ARM Cortex-A76 Technical Reference Manual](https://developer.arm.com/documentation/100798/latest/)
- [BCM2712 ARM Peripherals](https://datasheets.raspberrypi.com/bcm2712/bcm2712-peripherals.pdf)
- [RP1 Peripherals](https://datasheets.raspberrypi.com/rp1/rp1-peripherals.pdf)

### 技術参考資料
- Ubuntu linux-raspi カーネルソース
- [Rust Embedded Book](https://docs.rust-embedded.org/book/)
- [ARM64 Architecture Reference Manual](https://developer.arm.com/documentation/ddi0487/latest/)
- [GIC-400 Generic Interrupt Controller](https://developer.arm.com/documentation/ddi0471/latest/)

### 謝辞
- **Toshifumi NISHINAGA** (@tnishinaga) - pi5_hackプロジェクトの作者として、Pi5 ベアメタル開発の道筋を示していただきました
- Raspberry Pi Foundation - 優れたハードウェアプラットフォームの提供
- Rust Embedded Working Group - 組み込みシステム向けRustツールチェーンの開発
- ARM Limited - 包括的な技術ドキュメントの提供

---

**注意**: このプロジェクトは教育・研究目的で開発されています。本番環境での使用は推奨されません。