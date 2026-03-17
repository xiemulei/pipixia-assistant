# 皮皮虾助手 - 停止脚本

Write-Host "🦐 停止皮皮虾助手..." -ForegroundColor Cyan

# 停止后端进程 (rig-agent-scaffold)
Get-Process | Where-Object { $_.ProcessName -like "*rig-agent*" } | Stop-Process -Force -ErrorAction SilentlyContinue

# 停止前端进程 (Tauri)
Get-Process | Where-Object { $_.ProcessName -like "*rig-agent-ui*" -or $_.ProcessName -like "*pipixia*" } | Stop-Process -Force -ErrorAction SilentlyContinue

# 停止 Cargo 进程
Get-Process | Where-Object { $_.ProcessName -eq "cargo" } | Stop-Process -Force -ErrorAction SilentlyContinue

# 停止 Node 进程（可选，谨慎使用）
# Get-Process | Where-Object { $_.ProcessName -eq "node" } | Stop-Process -Force -ErrorAction SilentlyContinue

Write-Host "✅ 所有服务已停止" -ForegroundColor Green