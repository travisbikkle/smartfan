@echo off

chcp 65001 >nul

setlocal enabledelayedexpansion

REM 检查管理员权限
openfiles >nul 2>&1
if %errorlevel% neq 0 (
    echo 该脚本需要管理员权限
    echo 请右键选择"以管理员身份运行"
    timeout /t 30
    exit /b 1
)

REM 强制使用短路径解决中文问题
for /f "usebackq tokens=*" %%a in (`chdir`) do set "current_dir=%%~sa"

:menu
cls
echo 请选择安装模式。如果有ipmi驱动选1，如果没有驱动但是有BMC信息选2，两个都没有无法使用：
echo [1] in-band
echo [2] out-band
choice /c 12 /n /m "请选择安装模式(输入数字): "

if %errorlevel% equ 1 (
    set mode=in-band
) else if %errorlevel% equ 2 (
    set mode=out-band
    
    set /p "ip=请输入BMC IP地址："
    set /p "user=请输入BMC 用户名："
    set /p "pass=请输入BMC 密码："
    powershell -Command "(Get-Content '%current_dir%\HR650X.yaml') -replace 'host: .*', 'host: %ip%' -replace 'username: .*', 'username: %user%' -replace 'password: .*', 'password: %pass%' | Set-Content '%current_dir%\HR650X.yaml'"
)
    powershell -Command "(Get-Content '%current_dir%\HR650X.yaml') -replace 'host: .*', 'host: %ip%' -replace 'username: .*', 'username: %user%' -replace 'password: .*', 'password: %pass%' | Set-Content '%current_dir%\HR650X.yaml'"
) else (
    exit /b 1
)

REM 创建安装目录并复制文件
set "target_dir=%USERPROFILE%\smartfan"
robocopy "%current_dir%" "%target_dir%" *.* /E /COPYALL /XD "%current_dir%" /XF install.bat /NJH /NJS /NFL /NDL /NP
if %errorlevel% gtr 8 (
    echo [错误] 文件复制失败 代码: %errorlevel%
    echo 源路径: %current_dir%
    echo 目标路径: %target_dir%
    pause
    exit /b 1
)

REM 创建计划任务
set "exec_path=%target_dir%\smartfan.exe"
schtasks /create /tn "SmartFan" /tr "\"%exec_path%\" %mode%" /sc onstart /ru SYSTEM /rl highest /f
if %errorlevel% neq 0 (
    echo [错误] 计划任务创建失败 代码: %errorlevel%
    echo 可能原因:
    echo 1. 未使用管理员权限运行
    echo 2. 程序路径包含特殊字符
    echo 当前路径: %exec_path%
    pause
    exit /b 1
)

echo 安装完成，已设置为开机自动启动（%mode%模式）
pause
endlocal
