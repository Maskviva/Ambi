@echo off
chcp 65001 >nul 2>&1
setlocal enabledelayedexpansion

set "output_file=temp_combined.txt"
if exist "%output_file%" del "%output_file%"

for /r src %%f in (*.rs) do (
    echo. >> "%output_file%"
    echo ===== 文件：%%~nxf ===== >> "%output_file%"
    echo. >> "%output_file%"
    type "%%f" >> "%output_file%"
    echo. >> "%output_file%"
)

if not exist "%output_file%" (
    echo 错误：未找到任何 .rs 文件！
    pause
    exit /b 1
)

echo 正在复制到剪贴板...
clip < "%output_file%"
if errorlevel 1 (
    echo 错误：复制到剪贴板失败！
    pause
    exit /b 1
)

del "%output_file%"
echo 成功：所有 .rs 文件内容已复制到剪贴板！
pause
