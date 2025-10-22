# Set-PSReadLineOption -PredictionSource History
Set-PSReadlineKeyHandler -Key Tab -Function AcceptSuggestion
Set-PSReadLineKeyHandler -Chord "RightArrow" -Function ForwardWord
Set-PSReadlineKeyHandler -Key Ctrl+Backspace -Function BackwardKillWord
Set-PSReadlineKeyHandler -Key Ctrl+Delete -Function KillWord
fnm env --use-on-cd --shell powershell | Out-String | Invoke-Expression
