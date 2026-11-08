@echo off
rem Build the release binary and install it to a stable path (stops a running host first so the exe isn't locked).
set PATH=%PATH%;%USERPROFILE%\.cargo\bin
set BIN=%LOCALAPPDATA%\morc\bin
if not exist "%BIN%" mkdir "%BIN%"
if exist "%BIN%\morc.exe" "%BIN%\morc.exe" stop >nul 2>&1
timeout /t 1 /nobreak >nul
cd /d "%~dp0"
cargo build --release || exit /b 1
copy /y "%~dp0target\release\morc.exe" "%BIN%\morc.exe" >nul && echo installed %BIN%\morc.exe
