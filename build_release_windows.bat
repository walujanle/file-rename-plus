@echo off
setlocal enabledelayedexpansion

:: =============================================================================
:: File Rename Plus - Windows Build Script
:: =============================================================================

set "OUTPUT_DIR=build_release_windows"
set "EXE_NAME=file-rename-plus.exe"

echo.
echo ============================================
echo   File Rename Plus - Windows Build Script
echo ============================================
echo.

:: -----------------------------------------------------------------------------
:: Step 1: Check if Rust is installed
:: -----------------------------------------------------------------------------
echo [1/4] Checking Rust installation...

where rustc >nul 2>&1
if %errorlevel% neq 0 (
    echo.
    echo ERROR: Rust is not installed or not in PATH.
    echo Please install Rust from https://rustup.rs/
    echo.
    exit /b 1
)

for /f "tokens=*" %%i in ('rustc --version') do set RUST_VERSION=%%i
echo       Found: %RUST_VERSION%

where cargo >nul 2>&1
if %errorlevel% neq 0 (
    echo.
    echo ERROR: Cargo is not installed or not in PATH.
    echo Please install Rust from https://rustup.rs/
    echo.
    exit /b 1
)

for /f "tokens=*" %%i in ('cargo --version') do set CARGO_VERSION=%%i
echo       Found: %CARGO_VERSION%
echo.

:: -----------------------------------------------------------------------------
:: Step 2: Clean previous build output
:: -----------------------------------------------------------------------------
echo [2/4] Cleaning previous build...

if exist "%OUTPUT_DIR%" (
    rmdir /s /q "%OUTPUT_DIR%"
    echo       Removed existing %OUTPUT_DIR%
)
mkdir "%OUTPUT_DIR%"
echo       Created %OUTPUT_DIR%
echo.

:: -----------------------------------------------------------------------------
:: Step 3: Build release binary
:: -----------------------------------------------------------------------------
echo [3/4] Building release binary...
echo       This may take a few minutes...
echo.

cargo build --release
if %errorlevel% neq 0 (
    echo.
    echo ERROR: Build failed. Please check the errors above.
    exit /b 1
)

echo.
echo       Build completed successfully.
echo.

:: -----------------------------------------------------------------------------
:: Step 4: Copy build artifacts
:: -----------------------------------------------------------------------------
echo [4/4] Copying build artifacts...

if not exist "target\release\%EXE_NAME%" (
    echo.
    echo ERROR: Build artifact not found at target\release\%EXE_NAME%
    exit /b 1
)

copy "target\release\%EXE_NAME%" "%OUTPUT_DIR%\" >nul
echo       Copied %EXE_NAME%

if exist "README.md" (
    copy "README.md" "%OUTPUT_DIR%\" >nul
    echo       Copied README.md
)

if exist "LICENSE" (
    copy "LICENSE" "%OUTPUT_DIR%\" >nul
    echo       Copied LICENSE
)

echo.

:: -----------------------------------------------------------------------------
:: Summary
:: -----------------------------------------------------------------------------
echo ============================================
echo   Build Complete!
echo ============================================
echo.
echo   Output: %OUTPUT_DIR%\%EXE_NAME%
echo.

for %%A in ("%OUTPUT_DIR%\%EXE_NAME%") do set "FILE_SIZE=%%~zA"
set /a FILE_SIZE_KB=%FILE_SIZE% / 1024
set /a FILE_SIZE_MB=%FILE_SIZE_KB% / 1024

echo   Size: %FILE_SIZE_KB% KB (~%FILE_SIZE_MB% MB)
echo.
echo   The application is portable and requires
echo   no installation. Just run the .exe file.
echo.
echo ============================================
echo.

endlocal
exit /b 0
