# cargo-l1x (L1X VM Contract Builder (Cargo Templates))

Comprehensive contract development templates for the L1X Virtual Machine.

# Description

Build smart contracts quickly and efficiently on the L1X VM with modular and pre-configured Cargo templates. This repository includes fungible token (FT), non-fungible token (NFT), and other contract templates to streamline the development process. Ideal for developers new to L1X or looking for a faster way to prototype. Use the Contract Builder to compile and build contracts.

# Installation

**Install dependencies**

*Ubuntu 23.10*

```bash
sudo apt install clang llvm-15-dev libpolly-15-dev llvm-17 cmake
```

*Ubuntu 22.04*
```bash
sudo add-apt-repository 'deb http://apt.llvm.org/jammy/ llvm-toolchain-jammy-15 main' && \
(wget -O - https://apt.llvm.org/llvm-snapshot.gpg.key | sudo apt-key add - ) && \
sudo apt-get update && \
sudo apt-get install -y clang-15 llvm-15-dev libpolly-15-dev cmake

sudo add-apt-repository 'deb http://apt.llvm.org/jammy/ llvm-toolchain-jammy-17 main' && \
(wget -O - https://apt.llvm.org/llvm-snapshot.gpg.key | sudo apt-key add - ) && \
sudo apt-get update && \
sudo apt-get install -y llvm-17
```

*Ubuntu 20.04*
```bash
sudo add-apt-repository 'deb http://apt.llvm.org/focal/ llvm-toolchain-focal-15 main' && \
(wget -O - https://apt.llvm.org/llvm-snapshot.gpg.key | sudo apt-key add - ) && \
sudo apt-get update && \
sudo apt-get install -y clang-15 llvm-15-dev libpolly-15-dev cmake

sudo add-apt-repository 'deb http://apt.llvm.org/focal/ llvm-toolchain-focal-17 main' && \
(wget -O - https://apt.llvm.org/llvm-snapshot.gpg.key | sudo apt-key add - ) && \
sudo apt-get update && \
sudo apt-get install -y llvm-17
```

*Mac*

*[Install dependencies on Mac](https://l1x-sdk.gitbook.io/l1x-developer-interface/v/interface-essentials/l1x-vm-sdk/l1x-native-sdk-for-l1x-vm/set-up-environment/installation/install-cargo-l1x/mac-intel-and-silicon)*

**Add a compiler wasm32 target**
```bash
rustup target add wasm32-unknown-unknown
```

**Install `cargo-l1x`**
```bash
cargo install cargo-l1x --force
```

# Usage

**Create a project**
```bash
cargo l1x create some_project
```

**Create a project from a template**
```bash
cargo l1x create some_project --template ft
```
*List of available templates is here: https://github.com/L1X-Foundation/cargo-l1x-templates*

**Build the project**
```bash
cd some_project
cargo l1x build
```

**Clean the project**
```bash
cargo clean
```

**Help messages**

```bash
cargo l1x --help
cargo l1x build --help
cargo l1x create --help
```
