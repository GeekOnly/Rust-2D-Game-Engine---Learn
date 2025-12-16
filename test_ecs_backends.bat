@echo off
echo Testing ECS Backend System
echo ========================

echo.
echo 1. Listing available backends:
cargo run -p ecs --bin ecs_benchmark list

echo.
echo 2. Showing backend information:
cargo run -p ecs --bin ecs_benchmark info

echo.
echo 3. Testing backend functionality:
cargo run -p ecs --bin ecs_benchmark test

echo.
echo 4. Running quick benchmark (100 iterations):
cargo run -p ecs --bin ecs_benchmark benchmark --iterations 100 --warmup 10

echo.
echo 5. Running example:
cargo run -p ecs --example backend_chooser

echo.
echo ECS Backend System Test Complete!
pause