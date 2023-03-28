[27 Mar, 2023]

# Requirements

1. Replace all text instances of "RustDesk" to "CYMTV Remote" (such as window titles, "About" section, etc).
2. Replace all images/icons of RustDesk (https://play-lh.googleusercontent.com/wXeh3AaX496fjyuGl2mjfHYEGEvvvfkOBWSKf8KESKIUeohlEhU7VxvG_FRFoEmKa1A) with the attached "CYMTV-Remote-CYMR_Square Logo.png"
3. Install RustDesk Server on your Linux server and configure the client source code to use the RustDesk Server installed on your Linux server by default.

# Steps

1. Follow the steps at https://rustdesk.com/docs/en/dev/build/linux/ to setup the Linux environment.
2. Follow the steps at https://rustdesk.com/docs/en/dev/build/android/ to setup the Android environment.
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

# Build Linux App
1. install dependencies: 
```
sudo apt install -y g++ gcc git curl wget nasm yasm libgtk-3-dev clang libxcb-randr0-dev libxdo-dev libxfixes-dev libxcb-shape0-dev libxcb-xfixes0-dev libasound2-dev libpulse-dev cmake
```
2. install vcpkg
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
3. install Rust and Cargo
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```
4. install `sciter` library
```
sudo mkdir /usr/lib/rustdesk
wget https://raw.githubusercontent.com/c-smile/sciter-sdk/master/bin.lnx/x64/libsciter-gtk.so
sudo mv libsciter-gtk.so /usr/lib/rustdesk
``` 
5. to build Embedded UI / Enable Inline Builds: 
    - you need to have python3 installed
```
cd <source-code-directory>
python3 res/inline-sciter.py 
sed -i "s/default = \[\"use_dasp\"\]/default = \[\"use_dasp\",\"inline\"\]/g" Cargo.toml
```
6. build binary file
```
VCPKG_ROOT=$HOME/vcpkg cargo build --release

#move binary file to home 
mv target/release/rustdesk $HOME/CYMTV_Remote
```

# Build Android APK
- install Flutter 
```
#using snap
sudo snap install flutter --classic
flutter sdk-path
```
- install Rust and Cargo
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```
- install vcpkg
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
- install dependencies
```
sudo apt update -y
sudo apt-get -qq install -y g++ gcc git curl wget nasm yasm libgtk-3-dev clang cmake libclang-dev ninja-build llvm-dev libclang-10-dev llvm-10-dev pkg-config libxcb-randr0-dev libxdo-dev libxfixes-dev libxcb-shape0-dev libxcb-xfixes0-dev libasound2-dev libpulse-dev  libappindicator3-dev libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev libvdpau-dev libva-dev libclang-dev tree libc6-dev gcc-multilib g++-multilib openjdk-11-jdk-headless
```
- Installing Flutter Rust Bridge dependencies
```
cd <source-code-directory>
cargo install flutter_rust_bridge_codegen
pushd flutter && flutter pub get && popd
```
- Generating bridge files
```
#first make sure build is not inline
sed -i "s/default = \[\"use_dasp\",\"inline\"\]/default = \[\"use_dasp\"\]/g" Cargo.toml

~/.cargo/bin/flutter_rust_bridge_codegen --rust-input ./src/flutter_ffi.rs --dart-output ./flutter/lib/generated_bridge.dart
```
- Install active ffigen
```
dart pub global activate ffigen 5.0.1
```
- download additional dependencies
```
pushd /opt
sudo wget https://github.com/rustdesk/doc.rustdesk.com/releases/download/console/dep.tar.gz

sudo tar xzf dep.tar.gz
popd
```
- build rustdesk lib
```
rustup target add aarch64-linux-android
```
- install cargo-ndk 
```
cargo install cargo-ndk
```
- download Android NDK
    - you need to download and install Android studio first from [here](https://developer.android.com/studio)
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
- build rust for flutter library
```
./flutter/ndk_arm64.sh
```
- Moving generated library into jniLibs directory
```
mkdir -p ./flutter/android/app/src/main/jniLibs/arm64-v8a

cp ./target/aarch64-linux-android/release/liblibrustdesk.so ./flutter/android/app/src/main/jniLibs/arm64-v8a/librustdesk.so
```
- download necessary so files
```
pushd flutter
sudo wget -O so.tar.gz https://github.com/rustdesk/doc.rustdesk.com/releases/download/console/so.tar.gz

tar xzvf so.tar.gz
popd
```
- temporary use debug sign config
```
sed -i "s/signingConfigs.release/signingConfigs.debug/g" ./flutter/android/app/build.gradle
```
- build APK with Flutter
```
pushd flutter
flutter build apk --release --target-platform android-arm64 --split-per-abi
```
- move and rename APK file to home directory
```
mv build/app/outputs/flutter-apk/app-arm64-v8a-release.apk $HOME/CYMTV_Remote-release.apk
``` 