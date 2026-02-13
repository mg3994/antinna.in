@echo off
title Docker DB Up

echo ======================================
echo   Starting Database Containers
echo ======================================

REM Navigate to project directory
cd /d C:\Users\Manish\Desktop\workspace\docker\db

REM Check if Docker is running
docker info >nul 2>&1
if %errorlevel% neq 0 (
    echo.
    echo [ERROR] Docker Desktop is not running!
    echo Please start Docker Desktop and try again.
    echo.
    pause
    exit /b
)

echo.
echo Docker is running...
echo Starting containers...
echo.

docker-compose up -d

if %errorlevel% neq 0 (
    echo.
    echo [ERROR] Failed to start containers.
) else (
    echo.
    echo [SUCCESS] Database containers started successfully!
)

echo.
pause
