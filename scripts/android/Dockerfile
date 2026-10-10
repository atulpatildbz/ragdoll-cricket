# macroquad's APK builder ships Rust 1.79, too old for current fontdue and
# spade; layer a newer toolchain (with the Android target) on top.
FROM notfl3/cargo-apk
RUN rustup toolchain install 1.95.0 --profile minimal --target aarch64-linux-android \
    && rustup default 1.95.0
