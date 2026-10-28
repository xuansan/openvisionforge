param(
    [string]$OutputDirectory = "demo/calibration/chessboard"
)

Add-Type -AssemblyName System.Drawing

$outputPath = [System.IO.Path]::GetFullPath((Join-Path (Get-Location) $OutputDirectory))
[System.IO.Directory]::CreateDirectory($outputPath) | Out-Null

$canvasWidth = 960
$canvasHeight = 720
$columns = 10
$rows = 7
$square = 62
$angles = @(-13, -7, 0, 8, 14, -10, 11, 5)
$scales = @(0.86, 0.94, 1.00, 0.90, 0.82, 0.97, 0.88, 0.93)
$offsets = @(
    @(445, 340), @(500, 350), @(480, 360), @(430, 370),
    @(510, 330), @(460, 345), @(490, 375), @(440, 355)
)

for ($index = 0; $index -lt $angles.Count; $index++) {
    $bitmap = [System.Drawing.Bitmap]::new($canvasWidth, $canvasHeight)
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    try {
        $graphics.Clear([System.Drawing.Color]::White)
        $graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::None
        $graphics.TranslateTransform($offsets[$index][0], $offsets[$index][1])
        $graphics.RotateTransform($angles[$index])
        $graphics.ScaleTransform($scales[$index], $scales[$index])
        $originX = -($columns * $square) / 2
        $originY = -($rows * $square) / 2
        for ($row = 0; $row -lt $rows; $row++) {
            for ($column = 0; $column -lt $columns; $column++) {
                if (($row + $column) % 2 -eq 0) {
                    $graphics.FillRectangle(
                        [System.Drawing.Brushes]::Black,
                        $originX + $column * $square,
                        $originY + $row * $square,
                        $square,
                        $square
                    )
                }
            }
        }
    }
    finally {
        $graphics.Dispose()
    }
    try {
        $file = Join-Path $outputPath ("chessboard_{0:D2}.png" -f ($index + 1))
        $bitmap.Save($file, [System.Drawing.Imaging.ImageFormat]::Png)
    }
    finally {
        $bitmap.Dispose()
    }
}

Write-Output "Generated $($angles.Count) calibration fixtures in $outputPath"

$circlePath = [System.IO.Path]::GetFullPath((Join-Path (Get-Location) "demo/calibration/circle_grid"))
[System.IO.Directory]::CreateDirectory($circlePath) | Out-Null
$circleBitmap = [System.Drawing.Bitmap]::new($canvasWidth, $canvasHeight)
$circleGraphics = [System.Drawing.Graphics]::FromImage($circleBitmap)
try {
    $circleGraphics.Clear([System.Drawing.Color]::White)
    $circleGraphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $spacing = 70
    $diameter = 30
    $startX = 200
    $startY = 185
    for ($row = 0; $row -lt 6; $row++) {
        for ($column = 0; $column -lt 9; $column++) {
            $circleGraphics.FillEllipse(
                [System.Drawing.Brushes]::Black,
                $startX + $column * $spacing - $diameter / 2,
                $startY + $row * $spacing - $diameter / 2,
                $diameter,
                $diameter
            )
        }
    }
}
finally {
    $circleGraphics.Dispose()
}
try {
    $circleBitmap.Save((Join-Path $circlePath "circle_grid.png"), [System.Drawing.Imaging.ImageFormat]::Png)
}
finally {
    $circleBitmap.Dispose()
}

Write-Output "Generated circle grid fixture in $circlePath"
