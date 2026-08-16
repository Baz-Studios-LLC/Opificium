@echo off
REM Double-click to open the bench from source, on Windows.
REM
REM The counterpart to Opificium.command, which does the same on macOS. Neither
REM touches a version, a release or the launcher: they build whatever is in the
REM working tree and run that, so whichever branch is checked out is what opens.
REM
REM   Opificium.cmd                      the bench
REM   Opificium.cmd C:\path\to\game      that game as the project
REM
REM For one-click access, right-click this file and Send to > Desktop (create
REM shortcut). A shortcut on the desktop is not a file on the desktop.

REM Explorer hands a .cmd the folder it lives in already, but a shell may not,
REM and Bevy looks for assets\ beside the working directory - so this is set
REM explicitly or the bench opens with no fonts.
cd /d "%~dp0" || (echo Could not find the bench folder. & pause & exit /b 1)

REM rustup installs here, and it is not always on PATH.
set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
where cargo >nul 2>&1 || (echo Cargo is not on PATH. Install Rust from https://rustup.rs & pause & exit /b 1)

REM --release because the terrain bench meshes a world, and a debug build makes
REM that visibly slow. Only recompiles what changed.
cargo run --release -- %*

if errorlevel 1 pause
