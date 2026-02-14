@echo off
color 0F
echo ==================================================
echo      ARK COMPILER :: SYSTEM VERIFICATION
echo ==================================================
echo.
echo [1/2] Checking Python Environment...
python --version
if %ERRORLEVEL% NEQ 0 (
    color 4F
    echo [ERROR] Python not found!
    echo.
    pause
    exit /b 1
)

echo.
echo [2/2] Running THE GAUNTLET (Full Regression Suite)...
echo.
python meta/gauntlet.py
if %ERRORLEVEL% NEQ 0 (
    color 4F
    echo.
    echo ==================================================
    echo [FATAL] COMPILER CRASHED. DO NOT POST.
    echo ==================================================
    echo.
    pause
    exit /b 1
)

color 2F
echo.
echo ==================================================
echo [SUCCESS] SYSTEM VERIFIED. LIGHTS ARE GREEN.
echo ==================================================
echo.
echo You may now post "Victory".
echo.
pause
exit /b 0
