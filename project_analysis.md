# pi5_os プロジェクト解析

## システム概要

**pi5_os**は、Raspberry Pi 5上で動作するベアメタルOSの実装で、UNIXライクなシェル機能を提供します。OSやライブラリに依存せず、直接ハードウェアを制御するno_std環境で開発されており、プロセス管理、メモリ管理、ファイルシステム、割り込み処理などの基本的なカーネル機能を実装しています。ARMv8アーキテクチャ(AArch64)向けにRustで記述され、起動シーケンスにはARMアセンブリを使用しており、Raspberry Pi 5特有のRP1チップ経由のPL011 UARTやBCM2712システムタイマーなどのハードウェアを直接制御しています。

## 主要モジュール一覧

### コア機能
- **main.rs** - カーネルのエントリーポイントで、各モジュールの初期化とメインループを管理します。
- **startup.s** - ARM64アセンブリによる低レベル起動コードで、MMU設定やスタックポインタの初期化を行います。

### ハードウェア制御
- **uart.rs** - RP1チップ経由のPL011 UART通信を実装し、シリアル入出力を提供します。
- **gpio.rs** - Raspberry Pi 5のGPIOピン制御機能を提供し、デジタルI/Oを可能にします。
- **timer.rs** - BCM2712システムタイマーを制御し、時間管理とタイムスタンプ機能を提供します。
- **interrupt.rs** - 割り込みハンドラーとベクタテーブルを管理し、タイマーやUART割り込みを処理します。

### メモリとプロセス管理
- **mmu.rs** - メモリ管理ユニット(MMU)を制御し、ページテーブル設定とメモリ保護を実装します。
- **process.rs** - UNIX風のプロセス管理機能で、PID管理、親子関係、プロセスのライフサイクルを管理します。

### システムサービス
- **shell.rs** - UNIXライクなコマンドラインインターフェースを提供し、ユーザー入力を処理します。
- **filesystem.rs** - 基本的なファイルシステム機能で、ファイル操作やディレクトリ管理を実装します。
- **syscalls.rs** - システムコールインターフェースを提供し、ユーザーモードとカーネルモードの橋渡しをします。
- **signals.rs** - UNIXシグナルハンドリング機構を実装し、プロセス間通信をサポートします。
- **ipc.rs** - プロセス間通信(IPC)メカニズムを提供し、プロセス間のデータ交換を可能にします。
- **users.rs** - ユーザー管理機能を実装し、権限管理とユーザー認証を行います。
- **unix_commands.rs** - ls, cat, ps, echoなどのUNIX基本コマンドの実装を提供します。

### ビルドと設定
- **Cargo.toml** - Rustプロジェクト設定で、依存関係にtock-registers、cortex-a、heapless、defmtを使用します。
- **Makefile** - ビルドプロセスを自動化し、ELFからバイナリへの変換やSDカードイメージ作成を行います。
- **ldscript.lds** - リンカースクリプトで、0x200000番地からのメモリレイアウトを定義します。
- **config.txt** - Raspberry Pi 5のブート設定で、ARM64モードやUART有効化を指定します。

## 依存関係図

```mermaid
graph TD
    main[main.rs<br/>カーネルエントリーポイント] --> startup[startup.s<br/>起動シーケンス]
    main --> uart[uart.rs<br/>UART通信]
    main --> mmu[mmu.rs<br/>メモリ管理]
    main --> process[process.rs<br/>プロセス管理]
    main --> timer[timer.rs<br/>タイマー制御]
    main --> shell[shell.rs<br/>シェルUI]
    main --> interrupt[interrupt.rs<br/>割り込み処理]
    main --> gpio[gpio.rs<br/>GPIO制御]
    main --> filesystem[filesystem.rs<br/>ファイルシステム]
    main --> syscalls[syscalls.rs<br/>システムコール]
    main --> signals[signals.rs<br/>シグナル処理]
    main --> ipc[ipc.rs<br/>プロセス間通信]
    main --> users[users.rs<br/>ユーザー管理]
    
    shell --> unix_commands[unix_commands.rs<br/>UNIXコマンド]
    shell --> uart
    
    process --> syscalls
    process --> signals
    process --> ipc
    
    syscalls --> filesystem
    syscalls --> process
    
    interrupt --> timer
    interrupt --> uart
    
    unix_commands --> filesystem
    unix_commands --> process
    unix_commands --> users
    
    mmu --> startup
    
    style main fill:#ff9999
    style startup fill:#99ccff
    style uart fill:#99ff99
    style shell fill:#ffcc99
