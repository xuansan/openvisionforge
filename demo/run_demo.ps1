param(
  [string]$InputDir = "demo/input",
  [string]$OutputDir = "demo/output",
  [int]$Threshold = 128
)

Add-Type -AssemblyName System.Drawing

New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

$rows = New-Object System.Collections.Generic.List[string]
$rows.Add("file,width,height,black_pixels,white_pixels,black_ratio,white_ratio,output")

$files = Get-ChildItem -LiteralPath $InputDir -File | Where-Object {
  $_.Extension.ToLowerInvariant() -in @(".jpg", ".jpeg", ".png")
}

foreach ($file in $files) {
  $bitmap = [System.Drawing.Bitmap]::new($file.FullName)
  $width = $bitmap.Width
  $height = $bitmap.Height
  $out = [System.Drawing.Bitmap]::new($bitmap.Width, $bitmap.Height)
  $black = 0L
  $white = 0L

  for ($y = 0; $y -lt $bitmap.Height; $y++) {
    for ($x = 0; $x -lt $bitmap.Width; $x++) {
      $pixel = $bitmap.GetPixel($x, $y)
      $gray = [int](0.299 * $pixel.R + 0.587 * $pixel.G + 0.114 * $pixel.B)
      if ($gray -lt $Threshold) {
        $out.SetPixel($x, $y, [System.Drawing.Color]::Black)
        $black++
      } else {
        $out.SetPixel($x, $y, [System.Drawing.Color]::White)
        $white++
      }
    }
  }

  $outputPath = Join-Path $OutputDir ($file.BaseName + "_bw.png")
  $out.Save($outputPath, [System.Drawing.Imaging.ImageFormat]::Png)

  $bitmap.Dispose()
  $out.Dispose()

  $total = $black + $white
  $blackRatio = $black / $total
  $whiteRatio = $white / $total

  "{0}: {1}x{2}, black {3:P2}, white {4:P2} -> {5}" -f `
    $file.FullName, $width, $height, $blackRatio, $whiteRatio, $outputPath

  $rows.Add(("{0},{1},{2},{3},{4},{5:F6},{6:F6},{7}" -f `
    $file.FullName, $width, $height, $black, $white, $blackRatio, $whiteRatio, $outputPath))
}

$rows | Set-Content -LiteralPath (Join-Path $OutputDir "report.csv") -Encoding UTF8
