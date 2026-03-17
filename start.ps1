# 皮皮虾助手 - 启动脚本

Write-Host "🦐 启动皮皮虾助手..." -ForegroundColor Cyan

# 检查 Qdrant 是否运行
$qdrantRunning = Test-NetConnection -ComputerName localhost -Port 6333 -InformationLevel Quiet -WarningAction SilentlyContinue
if (-not $qdrantRunning) {
    Write-Host "⚠️  Qdrant 未运行，请先启动 Qdrant:" -ForegroundColor Yellow
    Write-Host "   docker run -p 6333:6333 -p 6334:6334 qdrant/qdrant" -ForegroundColor Gray
    Write-Host ""
    $continue = Read-Host "是否继续启动后端？(y/n)"
    if ($continue -ne "y") {
        exit 1
    }
}

# 设置环境变量（从 .env 文件读取或使用占位符）
if (Test-Path "$scriptDir\backend\.env") {
    Get-Content "$scriptDir\backend\.env" | ForEach-Object {
        if ($_ -match "^([^#][^=]+)=(.*)$") {
            [Environment]::SetEnvironmentVariable($matches[1].Trim(), $matches[2].Trim())
        }
    }
} else {
    Write-Host "⚠️  未找到 .env 文件，请复制 .env.example 并填入 API Key" -ForegroundColor Yellow
    Write-Host "   cp backend/.env.example backend/.env" -ForegroundColor Gray
    exit 1
}

# 获取脚本所在目录
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

# 启动后端
Write-Host "📦 启动后端服务..." -ForegroundColor Green
Set-Location "$scriptDir\backend"
Start-Process -FilePath "cargo" -ArgumentList "run" -NoNewWindow -PassThru | Out-Null

# 等待后端启动
Write-Host "⏳ 等待后端启动..." -ForegroundColor Gray
Start-Sleep -Seconds 5

# 检查后端是否启动成功
$backendRunning = Test-NetConnection -ComputerName localhost -Port 3000 -InformationLevel Quiet -WarningAction SilentlyContinue
if ($backendRunning) {
    Write-Host "✅ 后端启动成功: http://localhost:3000" -ForegroundColor Green
} else {
    Write-Host "⚠️  后端启动失败，请检查日志" -ForegroundColor Red
}

# 启动前端
Write-Host "🖥️  启动前端应用..." -ForegroundColor Green
Set-Location "$scriptDir\frontend"
Start-Process -FilePath "npm" -ArgumentList "run", "tauri", "dev" -NoNewWindow -PassThru | Out-Null

Write-Host ""
Write-Host "🎉 皮皮虾助手已启动！" -ForegroundColor Cyan
Write-Host "   后端: http://localhost:3000" -ForegroundColor Gray
Write-Host "   前端: Tauri 桌面窗口" -ForegroundColor Gray
Write-Host ""
Write-Host "💡 提示: 使用 .\stop.ps1 停止所有服务" -ForegroundColor Yellow