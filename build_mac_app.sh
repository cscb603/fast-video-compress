#!/bin/bash
# 星TAP 视频压缩 V4 - macOS App 打包脚本
# 星 TAP 实验室出品

set -e

APP_NAME="视频高速压缩"
VERSION="4.0.0"
APP_DIR="${APP_NAME}.app"
BINARY_NAME="VideoCompressor"

echo "🚀 开始构建 ${APP_NAME} v${VERSION}..."

# 1. 清理旧的构建
echo "🧹 清理旧的构建..."
rm -rf "${APP_DIR}"

# 2. 确保已经编译了 Release 版本
if [ ! -f "target/release/fast-video-compress-gui" ]; then
    echo "🦀 编译 Release 版本..."
    cargo build --release
fi

# 3. 创建 App Bundle 结构
echo "📦 创建 App Bundle 结构..."
mkdir -p "${APP_DIR}/Contents/MacOS"
mkdir -p "${APP_DIR}/Contents/Resources"

# 4. 拷贝二进制文件
echo "📋 拷贝二进制文件..."
cp "target/release/fast-video-compress-gui" "${APP_DIR}/Contents/MacOS/${BINARY_NAME}"
chmod +x "${APP_DIR}/Contents/MacOS/${BINARY_NAME}"

# 5. 拷贝图标资源
echo "🎨 拷贝图标资源..."

if [ -f "视频压缩图标.icns" ]; then
    cp "视频压缩图标.icns" "${APP_DIR}/Contents/Resources/AppIcon.icns"
    echo "✅ 使用 视频压缩图标.icns 图标"
elif [ -f "icon.icns" ]; then
    cp "icon.icns" "${APP_DIR}/Contents/Resources/AppIcon.icns"
    echo "✅ 使用 icon.icns 图标"
else
    echo "⚠️ 警告：未找到图标文件"
fi

# 6. 创建 Info.plist
echo "📝 创建 Info.plist..."
cat > "${APP_DIR}/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>${BINARY_NAME}</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon.icns</string>
    <key>CFBundleIdentifier</key>
    <string>com.xtap.video-compress</string>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>${VERSION}</string>
    <key>CFBundleVersion</key>
    <string>${VERSION}</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.15</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

# 7. 代码签名
echo "🔐 执行代码签名..."
codesign --force --deep --sign - "${APP_DIR}" || true

# 8. 清除扩展属性
echo "🧽 清除扩展属性..."
xattr -cr "${APP_DIR}" || true

# 9. 验证 App 结构
echo "✅ 验证 App 结构..."
echo "App Bundle 内容:"
ls -la "${APP_DIR}/Contents/MacOS/"
ls -la "${APP_DIR}/Contents/Resources/"

echo ""
echo "======================================"
echo "✅ 构建完成！"
echo "======================================"
echo "📱 App 位置：${APP_DIR}"
echo "🎉 可以使用了！"
echo ""
