@echo off
REM Run release exe
cd /d "%~dp0"
if exist "target\release\duplicates_resolver.exe" (
    target\release\duplicates_resolver.exe
) else (
    echo Error: Release executable not found at target\release\duplicates_resolver.exe
    exit /b 1
)
pause