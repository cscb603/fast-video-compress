#!/bin/bash
# 自动批量视频压缩 - 门禁自检脚本
# 集成 Ruff (格式化/检查), MyPy (类型检查), PyTest (单元测试)

set -e

echo "--- 1. Ruff 检查 ---"
ruff check . --fix
ruff format .

echo "--- 2. MyPy 类型检查 ---"
mypy src

echo "--- 3. PyTest 单元测试 ---"
pytest tests

echo "--- ✅ 所有检查通过！ ---"
