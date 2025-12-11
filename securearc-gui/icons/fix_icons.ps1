Add-Type -AssemblyName System.Drawing

$iconPath = "c:\Users\eliot\Documents\Projects\SecurityArc\securearc-gui\icons"
$files = @("32x32.png", "128x128.png", "128x128@2x.png", "icon.png")

foreach ($file in $files) {
    $fullPath = Join-Path $iconPath $file
    if (Test-Path $fullPath) {
        Write-Host "Converting $file to PNG..."
        try {
            # Load the image (it might be JPEG masquerading as PNG)
            $img = [System.Drawing.Image]::FromFile($fullPath)
            
            # Save to a temporary path as PNG
            $tempPath = $fullPath + ".temp.png"
            $img.Save($tempPath, [System.Drawing.Imaging.ImageFormat]::Png)
            $img.Dispose()

            # Replace original
            Move-Item -Force $tempPath $fullPath
            Write-Host "Successfully converted $file"
        } catch {
            Write-Error "Failed to convert ${file}: $_"
        }
    } else {
        Write-Warning "File not found: $file"
    }
}
