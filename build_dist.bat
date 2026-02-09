@echo off
setlocal

:: Configuration
set MAIN_SCRIPT=app_refactored.py
set OUTPUT_DIR=dist
set FFMPEG_BIN=C:\ffmpeg-8.0\bin
set APP_NAME="星TAP批量视频压缩win版"

echo [1/3] Preparing build directory...
:: Only clean if user wants, for now just ensure it exists
if not exist %OUTPUT_DIR% mkdir %OUTPUT_DIR%

echo [2/3] Building with Nuitka...
:: Using updated Nuitka flags
:: --standalone: Bundles python and deps into a folder
:: --windows-console-mode=disable: Hide console window (GUI app)
:: --plugin-enable=pyqt5: Handle PyQt5 plugins
:: --include-package=modules: Include our local modules
python -m nuitka ^
    --standalone ^
    --windows-console-mode=disable ^
    --plugin-enable=pyqt5 ^
    --include-package=modules ^
    --output-dir=%OUTPUT_DIR% ^
    %MAIN_SCRIPT%

if %errorlevel% neq 0 (
    echo Build Failed!
    pause
    exit /b %errorlevel%
)

echo [3/3] Copying FFmpeg binaries...
set DIST_ROOT=%OUTPUT_DIR%\app_refactored.dist

:: Check if FFmpeg exists
if exist "%FFMPEG_BIN%\ffmpeg.exe" (
copy "%FFMPEG_BIN%\ffmpeg.exe" "%DIST_ROOT%\"
copy "%FFMPEG_BIN%\ffprobe.exe" "%DIST_ROOT%\"
) else (
    echo WARNING: FFmpeg binaries not found at %FFMPEG_BIN%
    echo Please manually copy ffmpeg.exe and ffprobe.exe to dist\app_refactored.dist
)

:: Rename exe and folder to final product name
set NEW_NAME=星TAP批量视频压缩win版
if exist "%DIST_ROOT%\app_refactored.exe" (
    ren "%DIST_ROOT%\app_refactored.exe" "%NEW_NAME%.exe"
)

pushd %OUTPUT_DIR%
if exist "app_refactored.dist" (
    ren "app_refactored.dist" "%NEW_NAME%"
)
popd

echo.
echo Build Complete! Output folder: %OUTPUT_DIR%\%NEW_NAME%
echo You can now compile installer using setup.iss
pause
