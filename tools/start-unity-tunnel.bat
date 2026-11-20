@echo off
title Unity MCP Tunnel // Spark Studios x Veronika
color 0b

echo ========================================================
echo    SPARK STUDIOS - UNITY MCP ONE-CLICK TUNNEL
echo ========================================================
echo.

set CLOUDFLARED_EXE=%~dp0cloudflared.exe
set TOKEN=eyJhIjoiNDQyZjVlZGM1ZTI2NmMzZTgyNTAzZWM1ZjVhZDQwYTIiLCJ0IjoiNDAyZTVmNTAtOTc5MC00OTNhLTg4NzAtOTllNWIwZjRjZWZhIiwicyI6IkNtUHZQQ2lFclJiTTRnSmtCZjkwNEw1WjBySXM3V2F2ZmNHRjlpaGtiVmRrcXV3OUdKK1hmaDRaYm5PaGF3RmxnRW9mTVRLV28rdE1mN09EK2ExcVlnPT0ifQ==

if exist "%CLOUDFLARED_EXE%" goto RUN_TUNNEL

echo [1/2] Mendownload Cloudflared binary untuk Windows...
powershell -Command "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; (New-Object Net.WebClient).DownloadFile('https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-windows-amd64.exe', '%CLOUDFLARED_EXE%')"

if not exist "%CLOUDFLARED_EXE%" (
    echo [ERROR] Gagal mendownload cloudflared.exe. Pastikan koneksi internet aktif.
    pause
    exit /b 1
)

:RUN_TUNNEL
echo [2/2] Menghubungkan Tunnel ke Unity MCP (http://localhost:8080)...
echo.
echo ========================================================
echo STATUS: TUNNEL ONLINE!
echo Public Endpoint: https://unity.nyxstudios.site
echo Local Target   : http://localhost:8080
echo.
echo JANGAN TUTUP JENDELA INI SELAMA SEDANG MENGGUNAKAN MCP.
echo Tekan Ctrl+C untuk berhenti.
echo ========================================================
echo.

"%CLOUDFLARED_EXE%" tunnel run --token %TOKEN%
pause
