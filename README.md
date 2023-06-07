[27 Mar, 2023]

# Requirements

1. Replace all text instances of "RustDesk" to "CYMTV Remote" (such as window titles, "About" section, etc).
2. Replace all images/icons of RustDesk (https://play-lh.googleusercontent.com/wXeh3AaX496fjyuGl2mjfHYEGEvvvfkOBWSKf8KESKIUeohlEhU7VxvG_FRFoEmKa1A) with the attached "CYMTV-Remote-CYMR_Square Logo.png"
3. Install RustDesk Server on your Linux server and configure the client source code to use the RustDesk Server installed on your Linux server by default.

# Steps

1. Follow the steps at https://rustdesk.com/docs/en/dev/build/linux/ to setup the Linux environment or refer to steps below.
2. Follow the steps at https://rustdesk.com/docs/en/dev/build/android/ to setup the Android environment or refer to steps below.
3. Use [versacode autopatch](https://github.com/versacode/autopatch) to apply the source code changes automatically or upload the files in this archive if no changes to the referenced files were made after 27 March, 2023.
4. Install RustDesk hbbr and hbbs by executing the following commands:

```
wget https://github.com/rustdesk/rustdesk-server/releases/download/1.1.7-2/rustdesk-server-hbbr_1.1.7_amd64.deb

wget https://github.com/rustdesk/rustdesk-server/releases/download/1.1.7-2/rustdesk-server-hbbs_1.1.7_amd64.deb

dpkg -i rustdesk-server-hbbr_1.1.7_amd64.deb
dpkg -i rustdesk-server-hbbs_1.1.7_amd64.deb

systemctl enable rustdesk-hbbs.service
systemctl enable rustdesk-hbbr.service

service rustdesk-hbbr start
service rustdesk-hbbs start

#Remove unnecessary files
rm rustdesk-server-hbbr_1.1.7_amd64.deb
rm rustdesk-server-hbbs_1.1.7_amd64.deb
```

The RustDesk public key is automatically generated at `/var/lib/rustdesk-server/id*.pub`.

5. Ensure that the following ports are allowed through the firewall:

```
TCP: 21115
TCP: 21116
TCP: 21117
TCP: 21118
TCP: 21119
UDP: 21116
```

## Build Linux App

1. Install dependencies: 
```
sudo apt install -y g++ gcc git curl wget nasm yasm libgtk-3-dev clang libxcb-randr0-dev libxdo-dev libxfixes-dev libxcb-shape0-dev libxcb-xfixes0-dev libasound2-dev libpulse-dev cmake
```
2. Install vcpkg
```
cd $HOME
git clone https://github.com/microsoft/vcpkg
cd vcpkg
git checkout 2021.12.01
cd ..
vcpkg/bootstrap-vcpkg.sh
export VCPKG_ROOT=$HOME/vcpkg
vcpkg/vcpkg install libvpx libyuv opus
```
3. Install Rust and Cargo
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```
4. Install `sciter` library
```
sudo mkdir /usr/lib/rustdesk
wget https://raw.githubusercontent.com/c-smile/sciter-sdk/master/bin.lnx/x64/libsciter-gtk.so
sudo mv libsciter-gtk.so /usr/lib/rustdesk
``` 
5. To build Embedded UI / Enable Inline Builds: 
```
cd <source-code-directory>
python3 res/inline-sciter.py 
sed -i "s/default = \[\"use_dasp\"\]/default = \[\"use_dasp\",\"inline\"\]/g" Cargo.toml
```
6. Build binary file
```
VCPKG_ROOT=$HOME/vcpkg cargo build --release
#move binary file to home 
mv target/release/rustdesk $HOME/CYMTV_Remote
```

## Build Android APK

1. Install Flutter 
```
#using snap
sudo snap install flutter --classic
flutter sdk-path
```
2. Install Rust and Cargo
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```
3. Install vcpkg
```
cd $HOME
git clone https://github.com/microsoft/vcpkg
cd vcpkg
git checkout 2021.12.01
cd ..
vcpkg/bootstrap-vcpkg.sh
export VCPKG_ROOT=$HOME/vcpkg
vcpkg/vcpkg install libvpx libyuv opus
```
4. Install dependencies
```
sudo apt update -y
sudo apt-get -qq install -y g++ gcc git curl wget nasm yasm libgtk-3-dev clang cmake libclang-dev ninja-build llvm-dev libclang-10-dev llvm-10-dev pkg-config libxcb-randr0-dev libxdo-dev libxfixes-dev libxcb-shape0-dev libxcb-xfixes0-dev libasound2-dev libpulse-dev  libappindicator3-dev libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev libvdpau-dev libva-dev libclang-dev tree libc6-dev gcc-multilib g++-multilib openjdk-11-jdk-headless
```
5. Install Flutter Rust Bridge dependencies
```
cd <source-code-directory>
cargo install flutter_rust_bridge_codegen
pushd flutter && flutter pub get && popd
```
6. Generate bridge files
```
#first make sure build is not inline
sed -i "s/default = \[\"use_dasp\",\"inline\"\]/default = \[\"use_dasp\"\]/g" Cargo.toml

~/.cargo/bin/flutter_rust_bridge_codegen --rust-input ./src/flutter_ffi.rs --dart-output ./flutter/lib/generated_bridge.dart
```
7. Install active ffigen
```
dart pub global activate ffigen 5.0.1
```
8. Download additional dependencies
```
pushd /opt
sudo wget https://github.com/rustdesk/doc.rustdesk.com/releases/download/console/dep.tar.gz

sudo tar xzf dep.tar.gz
popd
```
9. Build rustdesk lib
```
rustup target add aarch64-linux-android
```
10. Install cargo-ndk 
```
cargo install cargo-ndk
```
11. Download Android NDK (download and install Android Studio first from https://developer.android.com/studio)
```
sudo wget -O android-ndk-r23c-linux.zip https://dl.google.com/android/repository/android-ndk-r23c-linux.zip

mkdir -p $HOME/Android/Sdk/ndk
mv android-ndk-r23c-linux.zip $HOME/Android/Sdk/ndk
pushd $HOME/Android/Sdk/ndk
unzip android-ndk-r23c-linux.zip 
rm android-ndk-r23c-linux.zip 

# you should add the following exports the rc file like ~/.bashrc
export ANDROID_NDK_HOME=$HOME/Android/Sdk/ndk/android-ndk-r23c
export PATH=$PATH:$HOME/Android/Sdk/ndk/android-ndk-r23c/prebuilt/linux-x86_64/bin

popd
```
12. build rust for flutter library
```
./flutter/ndk_arm64.sh
```
13. Move  generated library into `jniLibs` directory:
```
mkdir -p ./flutter/android/app/src/main/jniLibs/arm64-v8a

cp ./target/aarch64-linux-android/release/liblibrustdesk.so ./flutter/android/app/src/main/jniLibs/arm64-v8a/librustdesk.so
```
14. Download necessary `so` files
```
pushd flutter
sudo wget -O so.tar.gz https://github.com/rustdesk/doc.rustdesk.com/releases/download/console/so.tar.gz

tar xzvf so.tar.gz
popd
```
15. Build APK with Flutter
```
pushd flutter
flutter build apk --release --target-platform android-arm64 --split-per-abi
```
16. Move and rename APK file to home directory
```
mv build/app/outputs/flutter-apk/app-arm64-v8a-release.apk $HOME/CYMTV_Remote-release.apk
``` 

## Generate DEB file

1. Ensure that the correct binary is located at `binaries/linux/dpkg/cymtv-remote/usr/bin/cymtv-remote` then execute the following:

```
chmod 775 `binaries/linux/dpkg/cymtv-remote/DEBIAN/postinst
chmod 775 `binaries/linux/dpkg/cymtv-remote/DEBIAN/postrm
chmod 775 `binaries/linux/dpkg/cymtv-remote/DEBIAN/preinst
chmod 775 `binaries/linux/dpkg/cymtv-remote/DEBIAN/prerm
chmod 775 `binaries/linux/dpkg/cymtv-remote/usr/bin/cymtv-remote
```

2. Modify the file at `binaries/linux/dpkg/cymtv-remote/DEBIAN/control` to include the correct size, version and other necessary information.

3. Build the .deb file by running the following:
```
dpkg-deb --build `binaries/linux/dpkg/cymtv-remote`
```

## Generate Xcarchive/IPA

1. On Mac OS X, install `xcode` and add your developer account from "Settings > Accounts"
2. Download the following dependencies:
- https://github.com/rustdesk/doc.rustdesk.com/releases/download/console/ios_dep.tar.gz
- https://github.com/microsoft/vcpkg/archive/refs/heads/master.zip
3. Extract `ios_dep.tar.gz` and move its `vcpkg` into the home directory at `~`:

```
mv vcpkg ~
```

4. Extract `master.zip` and execute the following inside the `vcpkg` directory:
```
./bootstrap-vcpkg.sh
./vcpkg install aom:arm64-ios
```

5. While in the `vcpkg` directory, copy the files inside `installed/vcpkg/arm64-ios` to `~/vcpkg/installed`:

```
cp -R installed/vcpkg/arm64-ios ~/vcpkg/installed
```

6. Navigate to the project's directory and execute the following to compile an IPA for iOS:

```
VCPKG_ROOT=$HOME/vcpkg ./flutter/ios_arm64.sh
cd flutter
dart pub global activate ffigen
flutter build ipa --release --obfuscate --split-debug-info=./split-debug-info
# or use to compile xcarchive: flutter build ipa --release --obfuscate --split-debug-info=./split-debug-info --no-codesign
```

7. The IPA/xcarchive files are generated at `flutter/build/ios`.


## Generate Windows Executable

1. Download Visual Studo Community for C++ (https://visualstudio.microsoft.com/), Rust (https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe) and vcpkg from https://github.com/microsoft/vcpkg/archive/refs/heads/master.zip
2. Extract `master.zip` and navigate to `vcpkg` then execute the following:

```
./bootstrap-vcpkg.bat
cd ..
vcpkg\vcpkg install libvpx:x64-windows-static libyuv:x64-windows-static opus:x64-windows-static aom:x64-windows-static
```
3. Download and install LLVM: https://github.com/llvm/llvm-project/releases (or 64-bit binary: https://github.com/llvm/llvm-project/releases/download/llvmorg-15.0.2/LLVM-15.0.2-win64.exe)

4. Navigate to Control Panel > System > Edit the system environment variables > Environment Variables... > System variables > New:

```
Variable name: VCPKG_ROOT
Variable value: [absolute path to vcpkg directory extracted in #2] (example: C:\vcpkg)

Variable name: LIBCLANG_PATH
Variable value: [LLVM installation path in #3]\bin (example C:\Program Files\LLVM\bin)
```

5. Download `sciter.dll` (direct link: https://raw.githubusercontent.com/c-smile/sciter-sdk/master/bin.win/x64/sciter.dll) and move it inside the project's directory at `target/debug` then execute `cargo run` to compile/run the debug version:

```
#cd into the project's directory
mkdir -p target\debug
#Download https://raw.githubusercontent.com/c-smile/sciter-sdk/master/bin.win/x64/sciter.dll
mv sciter.dll target/debug
cargo run
```

6. In `Cargo.toml` file, replace the following:

```
default = ["use_dasp"] 
```
with:

```
default = ["use_dasp", "inline"] 
```

and then run `python res/inline-sciter.py` and `cargo build --release` to generate the executable file, which will be built at `target/release`.