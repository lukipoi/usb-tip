@echo off
rem USB TIP 启动器（pythonw => 不弹控制台窗口）
setlocal
set "PYW=pythonw"
where pythonw >nul 2>nul || set "PYW=python"
start "" "%PYW%" "%~dp0py\usbtip.py" %*
endlocal
