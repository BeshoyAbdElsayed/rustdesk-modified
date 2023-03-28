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