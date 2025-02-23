# Tauri App Readme

This document provides a comprehensive guide to setting up and running your Rust Tauri application on Windows. It includes installation instructions for all necessary dependencies and steps to start the development server.

## Prerequisites

Before you begin, ensure your system meets the following requirements:

- **Operating System**: Windows
- **Development Tools**: Microsoft C++ Build Tools, Microsoft Edge WebView2
- **Programming Languages**: Rust, Node.js (optional for JavaScript frontend)

## Installation Instructions

### 1. Microsoft C++ Build Tools

Tauri requires the Microsoft C++ Build Tools for development. Follow these steps to install:

1. **Download** the Microsoft C++ Build Tools installer from the official Microsoft website.
2. **Run** the installer and select the “Desktop development with C++” option during installation.

### 2. Rust Installation

Tauri is built with Rust. Install Rust by following these steps:

1. Visit the official Rust installation page at [rust-lang.org/tools/install](https://www.rust-lang.org/tools/install).
2. Alternatively, use `winget` to install `rustup` via PowerShell:
   ```powershell
   winget install --id Rustlang.Rustup
   ```
3. Ensure the MSVC toolchain is set as the default:
    - During installation, select the MSVC Rust toolchain as the default host triple (e.g., `x86_64-pc-windows-msvc`).
    - If Rust is already installed, set the MSVC toolchain as default by running:
      ```powershell
      rustup default stable-msvc
      ```

### 3. Node.js Installation (Optional)

If you plan to use a JavaScript frontend framework, install Node.js:

1. **Download** the Long Term Support (LTS) version from the [Node.js website](https://nodejs.org/).
2. **Install** Node.js and verify the installation:
   ```powershell
   node -v
   npm -v
   ```
3. **Restart** your Terminal or computer to apply changes.
4. Optionally, enable other package managers like `pnpm` or `yarn`:
   ```powershell
   corepack enable
   ```

### 4. OpenCV Installation (Optional)

For applications requiring OpenCV, install it via Chocolatey or vcpkg:

- **Chocolatey**:
  ```powershell
  choco install llvm opencv
  ```
  Set the following environment variables:
    - `OPENCV_LINK_LIBS`
    - `OPENCV_LINK_PATHS`
    - `OPENCV_INCLUDE_PATHS`

- **vcpkg**:
  ```powershell
  vcpkg install llvm opencv4[contrib,nonfree]
  ```
  Set the environment variable `VCPKGRS_DYNAMIC` to `"1"` unless targeting a static build.

## Starting the Development Server

Once all dependencies are installed, follow these steps to start your Tauri app:

1. **Navigate** to the project directory
2. **Install** dependencies using your preferred package manager:
   ```powershell
   pnpm install
   ```
3. **Start** the Tauri development server:
   ```powershell
   pnpm tauri dev
   ```
4. A new window will open displaying your running Tauri application.

## Conclusion

Congratulations! You have successfully set up and launched your Tauri application on Windows. For further customization and development, refer to the [Tauri documentation](https://tauri.app/).

Happy coding! 🚀


install last version of clang + llvm 
https://github.com/llvm/llvm-project/releases/tag/llvmorg-19.1.7


add them to path 

LLVM_CONFIG_PATH
LIBCLANG_PATH