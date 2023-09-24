@echo off
:: Build the project in release mode
cargo build --release
:: Move the executable to the root directory
move target\release\iron_ssg.exe .\
:: Provide a completion message
echo Build completed and moved to root directory.
exit /b 0
