@echo off
title Docker DB Down

echo ======================================
echo   Stopping Database Containers
echo ======================================

REM Navigate to project directory
cd /d C:\Users\Manish\Desktop\workspace\docker\db

REM Check if Docker is running
docker info >nul 2>&1
if %errorlevel% neq 0 (
    echo.
    echo [ERROR] Docker Desktop is not running!
    echo Nothing to stop.
    echo.
    pause
    exit /b
)

echo.
echo Docker is running...
echo Stopping containers...
echo.

docker-compose down

if %errorlevel% neq 0 (
    echo.
    echo [ERROR] Failed to stop containers.
) else (
    echo.
    echo [SUCCESS] Database containers stopped successfully!
)

echo.
pause
