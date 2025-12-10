Note this is only tested using:

- Apple M4 with macOS Ventura 15.7 (24G222)
- [esp32-s3-n16r8](/docs/device.avif)

![device](docs/device.avif)

---

This git repo was initialed by:

```shell
cargo install esp-generate
esp-generate --chip=esp32s3 esp32-s3-n16r8
```

1. [init.sh](/scripts/init.sh) should help you install some non project level dependencies.
2. [deploy.sh](/scripts/deploy.sh) should help you build the project and deploy the binary to your esp32s3 device.


References:

1. https://github.com/esp-rs/esp-hal
2. https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description
3. https://esp32.implrust.com/embassy/blinky-with-embassy.html
4. https://github.com/esp-rs/esp-generate
5. https://esp32.implrust.com/dev-env.html
6. https://github.com/esp-rs/espup
7. https://github.com/esp-rs/espflash/blob/main/espflash/README.md#installation
8. https://github.com/esp-rs/awesome-esp-rust
9. https://docs.espressif.com/projects/esp-idf/en/stable/esp32s3/get-started/linux-macos-setup.html
10. https://www.jetbrains.com/help/rust/rust-toolchain.html#wsl
11. https://fcc.report/FCC-ID/2BB77-ESPS3-32E
12. https://github.com/microrobotics/ESP32-S3-N16R8/blob/main/ESP32-S3-N16R8_User_Guide.pdf
13. https://documentation.espressif.com/esp32-s3-wroom-1_wroom-1u_datasheet_en.pdf
14. https://device.report/fccid/2BB77ESPS332E
