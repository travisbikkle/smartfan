@echo off
setlocal

REM Check if running as administrator
openfiles >nul 2>&1
if %errorlevel% neq 0 (
    echo This script requires administrator privileges.
    echo Please run this script as an administrator.
    pause
    exit /b 1
)

REM Get the current directory
set current_path=%~dp0

REM Create a task in Task Scheduler to run smartfan.exe with in-band parameter at startup
schtasks /create /tn "SmartFan" /tr "%current_path%smartfan.exe in-band" /sc onstart /rl highest /f

REM Copy HR650X.yaml to the same directory as smartfan.exe
copy "%current_path%HR650X.yaml" "%current_path%smartfan.exe"

echo SmartFan has been installed and will run at startup with in-band parameter.
pause
endlocal
