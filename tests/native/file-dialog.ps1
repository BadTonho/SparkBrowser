param(
  [Parameter(Mandatory = $true)][string]$Path,
  [int]$TimeoutSeconds = 20
)

# Fills in and confirms the native "Open" file dialog of the application (Windows only), so a test
# can pick a file the way a person would. It exits 0 once the dialog has accepted the file and closed.
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
Add-Type -AssemblyName System.Windows.Forms

$root = [System.Windows.Automation.AutomationElement]::RootElement
$dialogClass = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ClassNameProperty, '#32770')
# 1148 is the "File name" box of the common file dialog.
$fileNameBox = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::AutomationIdProperty, '1148')

function Find-FileNameBox {
  foreach ($window in $root.FindAll([System.Windows.Automation.TreeScope]::Children, $dialogClass)) {
    $box = $window.FindFirst([System.Windows.Automation.TreeScope]::Descendants, $fileNameBox)
    if ($box) { return $box }
  }
  return $null
}

$deadline = (Get-Date).AddSeconds($TimeoutSeconds)
$edit = $null
while ((Get-Date) -lt $deadline -and -not $edit) {
  $edit = Find-FileNameBox
  if (-not $edit) { Start-Sleep -Milliseconds 200 }
}
if (-not $edit) { Write-Error 'The file dialog did not appear'; exit 2 }

# The file name box has the focus when the dialog opens; try to focus its text field to be sure.
$field = $edit.FindFirst([System.Windows.Automation.TreeScope]::Descendants,
  (New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty, [System.Windows.Automation.ControlType]::Edit)))
foreach ($target in @($field, $edit)) {
  if ($target) { try { $target.SetFocus(); break } catch {} }
}
Start-Sleep -Milliseconds 300

# Type the file name like a person would, then press Enter (the same as pressing "Open").
$escaped = [regex]::Replace($Path, '[+^%~(){}\[\]]', '{$0}')
[System.Windows.Forms.SendKeys]::SendWait('^a' + $escaped + '{ENTER}')

# Closed once no file dialog with a file name box is left (looking again, so a stale element cannot answer).
$until = (Get-Date).AddSeconds(8)
while ((Get-Date) -lt $until) {
  if (-not (Find-FileNameBox)) { exit 0 }
  Start-Sleep -Milliseconds 200
}
Write-Error 'The file dialog stayed open'
exit 4
