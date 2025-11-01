# --- Modules
Import-Module PSReadLine
Import-Module Terminal-Icons -ErrorAction SilentlyContinue
# Import-Module PSFzf -ErrorAction SilentlyContinue
# Enable-PsFzfAliases

# --- PSReadLine Settings
Set-PSReadLineOption -PredictionSource HistoryAndPlugin
Set-PSReadLineOption -PredictionViewStyle InlineView

# --- Key Bindings
Set-PSReadLineKeyHandler -Key Tab -Function MenuComplete
Set-PSReadLineKeyHandler -Key Shift+Tab -Function AcceptSuggestion
Set-PSReadLineKeyHandler -Key Ctrl+Backspace -Function BackwardKillWord
Set-PSReadLineKeyHandler -Key Ctrl+Delete -Function KillWord
#Set-PSReadLineKeyHandler -Key Alt+RightArrow -Function ForwardWord
#Set-PSReadLineKeyHandler -Key Alt+LeftArrow -Function BackwardWord

# --- Environment Setup
fnm env --use-on-cd --shell powershell | Out-String | Invoke-Expression

# --- Aliases and Functions
function ll {
    Get-ChildItem -Force | Select-Object Mode, LastWriteTime,
        @{Name='Size';Expression={"{0:N1} KB" -f ($_.Length / 1KB)}},
        Name | Format-Table -AutoSize
}
