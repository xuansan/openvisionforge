param(
    [string]$Filter = "*",
    [string]$OutputRoot = "records/node_smoke",
    [string]$SampleImage = "demo/input/d34084cc1f765c877abcdf2d515cacf3.jpg",
    [switch]$Build,
    [switch]$FailOnError
)

$ErrorActionPreference = "Stop"

function Write-Utf8NoBom {
    param(
        [Parameter(Mandatory = $true)][string]$Path,
        [Parameter(Mandatory = $true)][string]$Text
    )
    $encoding = New-Object System.Text.UTF8Encoding($false)
    [System.IO.File]::WriteAllText($Path, $Text, $encoding)
}

function Safe-Name {
    param([Parameter(Mandatory = $true)][string]$Value)
    return ($Value -replace "[^a-zA-Z0-9_.-]", "_")
}

function Add-Node {
    param(
        [Parameter(Mandatory = $true)][ref]$Nodes,
        [Parameter(Mandatory = $true)][string]$Id,
        [Parameter(Mandatory = $true)][string]$Operator,
        [Parameter(Mandatory = $true)]$Params
    )
    $Nodes.Value += @([ordered]@{
        id       = $Id
        operator = $Operator
        params   = $Params
    })
}

function Add-Edge {
    param(
        [Parameter(Mandatory = $true)][ref]$Edges,
        [Parameter(Mandatory = $true)][string]$FromNode,
        [Parameter(Mandatory = $true)][string]$FromPort,
        [Parameter(Mandatory = $true)][string]$ToNode,
        [Parameter(Mandatory = $true)][string]$ToPort
    )
    $Edges.Value += @([ordered]@{
        from_node = $FromNode
        from_port = $FromPort
        to_node   = $ToNode
        to_port   = $ToPort
    })
}

function Param-Value {
    param(
        [Parameter(Mandatory = $true)]$Param,
        [Parameter(Mandatory = $true)][string]$OperatorId,
        [Parameter(Mandatory = $true)][string]$OutputDir,
        [Parameter(Mandatory = $true)][string]$SampleImagePath,
        [Parameter(Mandatory = $true)][string]$SampleJsonPath
    )

    $key = [string]$Param.key
    $kind = [string]$Param.kind
    $default = $Param.default

    if ($null -ne $default) {
        if ($kind -eq "Number") {
            $number = 0.0
            if ([double]::TryParse([string]$default, [ref]$number)) {
                return $number
            }
        }
        if ($kind -eq "Boolean") {
            return (([string]$default).ToLowerInvariant() -eq "true")
        }
        if ($kind -eq "Json") {
            try {
                return ([string]$default | ConvertFrom-Json)
            } catch {
                return @{}
            }
        }
        return $default
    }

    if ($key -match "model") { return "models/yolo11n.onnx" }
    if ($key -match "image|input" -and $key -match "path") { return $SampleImagePath }
    if ($key -eq "path") {
        if ($OperatorId -match "read") { return $SampleJsonPath }
        return (Join-Path $OutputDir ((Safe-Name $OperatorId) + ".json"))
    }
    if ($key -match "output|manifest|report|csv") {
        return (Join-Path $OutputDir ((Safe-Name $OperatorId) + ".json"))
    }
    if ($key -match "root|folder|directory") { return $OutputDir }
    if ($key -match "url") { return "http://127.0.0.1:9/test" }
    if ($key -match "endpoint|broker|server|host") { return "127.0.0.1:9" }
    if ($key -match "bucket") { return "vision-test" }
    if ($key -match "key") { return ((Safe-Name $OperatorId) + ".json") }
    if ($key -match "database|db") { return "vision_test" }
    if ($key -match "table|collection|topic|queue|channel|method|service") { return "vision_test" }
    if ($key -match "user") { return "vision" }
    if ($key -match "password|token|secret") { return "test" }
    if ($key -match "class|label") { return "part" }
    if ($key -match "expected") {
        if ($kind -eq "Boolean") { return $true }
        return "OK"
    }

    if ($kind -eq "Number") { return 1 }
    if ($kind -eq "Boolean") { return $false }
    if ($kind -eq "Json") { return @{} }
    if ($kind -eq "Enum" -and $Param.options.Count -gt 0) { return $Param.options[0] }
    return "test"
}

function Params-For {
    param(
        [Parameter(Mandatory = $true)]$Operator,
        [Parameter(Mandatory = $true)][string]$OutputDir,
        [Parameter(Mandatory = $true)][string]$SampleImagePath,
        [Parameter(Mandatory = $true)][string]$SampleJsonPath
    )

    $params = [ordered]@{}
    foreach ($param in @($Operator.params)) {
        $params[$param.key] = Param-Value $param $Operator.id $OutputDir $SampleImagePath $SampleJsonPath
    }

    if ($Operator.id -eq "opencv.image.read") {
        $params["path"] = $SampleImagePath
    }
    if ($Operator.id -eq "opencv.image.write") {
        $params["output"] = (Join-Path $OutputDir "opencv_image_write.png")
    }
    if ($Operator.id -match "infer\.ultralytics|infer\.yolo|infer\.onnxruntime") {
        $params["backend"] = "mock"
        $params["model_path"] = "models/yolo11n.onnx"
        $params["labels"] = @("part", "defect")
        $params["score_threshold"] = 0.1
    }
    if ($Operator.id -match "^comm\.") {
        $params["mode"] = "outbox"
    }

    return $params
}

function Add-Provider {
    param(
        [Parameter(Mandatory = $true)][string]$Type,
        [Parameter(Mandatory = $true)][ref]$Nodes,
        [Parameter(Mandatory = $true)][ref]$Edges,
        [Parameter(Mandatory = $true)][string]$Prefix,
        [Parameter(Mandatory = $true)][string]$SampleImagePath,
        [Parameter(Mandatory = $true)][string]$SampleJsonPath
    )

    switch ($Type) {
        "image" {
            $id = "${Prefix}_image_read"
            Add-Node $Nodes $id "opencv.image.read" ([ordered]@{ path = $SampleImagePath })
            return @{ node = $id; port = "image" }
        }
        "mask" {
            $source = Add-Provider "image" $Nodes $Edges "${Prefix}_mask" $SampleImagePath $SampleJsonPath
            $id = "${Prefix}_otsu"
            Add-Node $Nodes $id "opencv.threshold.otsu" ([ordered]@{})
            Add-Edge $Edges $source.node $source.port $id "image"
            return @{ node = $id; port = "mask" }
        }
        "regions" {
            $source = Add-Provider "mask" $Nodes $Edges "${Prefix}_regions" $SampleImagePath $SampleJsonPath
            $id = "${Prefix}_blob"
            Add-Node $Nodes $id "opencv.blob.detect.mock" ([ordered]@{})
            Add-Edge $Edges $source.node $source.port $id "mask"
            return @{ node = $id; port = "regions" }
        }
        "contours" {
            $source = Add-Provider "image" $Nodes $Edges "${Prefix}_contours" $SampleImagePath $SampleJsonPath
            $id = "${Prefix}_find_contours"
            Add-Node $Nodes $id "opencv.analysis.find_contours" ([ordered]@{ min_area = 1; max_contours = 100 })
            Add-Edge $Edges $source.node $source.port $id "image"
            return @{ node = $id; port = "contours" }
        }
        "detections" {
            $source = Add-Provider "image" $Nodes $Edges "${Prefix}_det" $SampleImagePath $SampleJsonPath
            $id = "${Prefix}_yolo_mock"
            Add-Node $Nodes $id "dl.yolo.detect.mock" ([ordered]@{ score_threshold = 0.1 })
            Add-Edge $Edges $source.node $source.port $id "image"
            return @{ node = $id; port = "detections" }
        }
        "measurements" {
            $source = Add-Provider "contours" $Nodes $Edges "${Prefix}_meas" $SampleImagePath $SampleJsonPath
            $id = "${Prefix}_area"
            Add-Node $Nodes $id "opencv.measure.area" ([ordered]@{})
            Add-Edge $Edges $source.node $source.port $id "contours"
            return @{ node = $id; port = "measurements" }
        }
        "rule" {
            $source = Add-Provider "any" $Nodes $Edges "${Prefix}_rule" $SampleImagePath $SampleJsonPath
            $id = "${Prefix}_rule_bool"
            Add-Node $Nodes $id "rule.bool" ([ordered]@{ expected = $true })
            Add-Edge $Edges $source.node $source.port $id "data"
            return @{ node = $id; port = "rule" }
        }
        "bool" {
            $source = Add-Provider "rule" $Nodes $Edges "${Prefix}_bool" $SampleImagePath $SampleJsonPath
            $id = "${Prefix}_not"
            Add-Node $Nodes $id "logic.not" ([ordered]@{})
            Add-Edge $Edges $source.node $source.port $id "rule"
            return @{ node = $id; port = "passed" }
        }
        "text" {
            $source = Add-Provider "rule" $Nodes $Edges "${Prefix}_text" $SampleImagePath $SampleJsonPath
            $id = "${Prefix}_final"
            Add-Node $Nodes $id "logic.final_result" ([ordered]@{})
            Add-Edge $Edges $source.node $source.port $id "rule"
            return @{ node = $id; port = "status" }
        }
        "trigger" {
            $id = "${Prefix}_trigger"
            Add-Node $Nodes $id "trigger.software.mock" ([ordered]@{})
            return @{ node = $id; port = "trigger" }
        }
        "capture" {
            $source = Add-Provider "trigger" $Nodes $Edges "${Prefix}_cap" $SampleImagePath $SampleJsonPath
            $id = "${Prefix}_camera"
            Add-Node $Nodes $id "source.camera.mock" ([ordered]@{})
            Add-Edge $Edges $source.node $source.port $id "trigger"
            return @{ node = $id; port = "capture" }
        }
        "location" {
            $source = Add-Provider "image" $Nodes $Edges "${Prefix}_loc" $SampleImagePath $SampleJsonPath
            $id = "${Prefix}_loc_blob"
            Add-Node $Nodes $id "opencv.locate.blob" ([ordered]@{})
            Add-Edge $Edges $source.node $source.port $id "image"
            return @{ node = $id; port = "location" }
        }
        "json" {
            $source = Add-Provider "contours" $Nodes $Edges "${Prefix}_json" $SampleImagePath $SampleJsonPath
            return @{ node = $source.node; port = "stats" }
        }
        "final" {
            $source = Add-Provider "rule" $Nodes $Edges "${Prefix}_finalsrc" $SampleImagePath $SampleJsonPath
            $id = "${Prefix}_final_result"
            Add-Node $Nodes $id "logic.final_result" ([ordered]@{})
            Add-Edge $Edges $source.node $source.port $id "rule"
            return @{ node = $id; port = "final" }
        }
        default {
            $id = "${Prefix}_any"
            Add-Node $Nodes $id "storage.local.read" ([ordered]@{ path = $SampleJsonPath })
            return @{ node = $id; port = "data" }
        }
    }
}

$projectRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
Set-Location $projectRoot

if ($Build) {
    cargo build -p vision-cli
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

$exe = Join-Path $projectRoot "target/debug/vision-cli.exe"
if (!(Test-Path $exe)) {
    cargo build -p vision-cli
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

$outputRootPath = Join-Path $projectRoot $OutputRoot
$workflowDir = Join-Path $outputRootPath "workflows"
$artifactDir = Join-Path $outputRootPath "outputs"
$artifactParamDir = Join-Path $OutputRoot "outputs"
New-Item -ItemType Directory -Force -Path $workflowDir, $artifactDir | Out-Null

$sampleJson = Join-Path $artifactDir "sample.json"
Write-Utf8NoBom $sampleJson (@{
        ok    = $true
        count = 2
        score = 0.95
        label = "part"
        value = 42
        text  = "OK"
        bbox  = @{ x = 10; y = 10; width = 20; height = 30 }
    } | ConvertTo-Json -Depth 8)

$schema = & $exe export-schema | ConvertFrom-Json
$operators = @($schema | Where-Object { $_.id -like $Filter })

$results = @()
foreach ($operator in $operators) {
    $safe = Safe-Name $operator.id
    $nodes = @()
    $edges = @()

    foreach ($input in @($operator.input_ports)) {
        $provider = Add-Provider ([string]$input.value_type) ([ref]$nodes) ([ref]$edges) ("p" + $nodes.Count) $SampleImage $sampleJson
        Add-Edge ([ref]$edges) $provider.node $provider.port "target" $input.name
    }

    Add-Node ([ref]$nodes) "target" $operator.id (Params-For $operator $artifactParamDir $SampleImage $sampleJson)

    $workflow = [ordered]@{
        id      = "smoke_$safe"
        name    = "smoke $($operator.id)"
        version = "0.1.0"
        nodes   = $nodes
        edges   = $edges
    }

    $workflowPath = Join-Path $workflowDir ($safe + ".json")
    Write-Utf8NoBom $workflowPath ($workflow | ConvertTo-Json -Depth 30)

    $watch = [Diagnostics.Stopwatch]::StartNew()
    $previousErrorActionPreference = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    $output = & $exe run $workflowPath 2>&1
    $exitCode = $LASTEXITCODE
    $ErrorActionPreference = $previousErrorActionPreference
    $watch.Stop()

    $results += [pscustomobject]@{
        id        = $operator.id
        category  = $operator.category
        name      = $operator.name
        status    = if ($exitCode -eq 0) { "PASS" } else { "FAIL" }
        exit_code = $exitCode
        ms        = [math]::Round($watch.Elapsed.TotalMilliseconds, 1)
        workflow  = $workflowPath
        message   = (($output | Select-Object -Last 10) -join " ")
    }
}

$csvPath = Join-Path $outputRootPath "node_smoke_results.csv"
$jsonPath = Join-Path $outputRootPath "node_smoke_results.json"
$summaryPath = Join-Path $outputRootPath "node_smoke_summary.json"

$results | Export-Csv -NoTypeInformation -Encoding UTF8 $csvPath
Write-Utf8NoBom $jsonPath ($results | ConvertTo-Json -Depth 8)

$summary = [ordered]@{
    total       = $results.Count
    pass        = @($results | Where-Object { $_.status -eq "PASS" }).Count
    fail        = @($results | Where-Object { $_.status -eq "FAIL" }).Count
    filter      = $Filter
    csv         = $csvPath
    json        = $jsonPath
    workflows   = $workflowDir
    generatedAt = (Get-Date).ToString("s")
}
Write-Utf8NoBom $summaryPath ($summary | ConvertTo-Json -Depth 6)

Write-Host "TOTAL=$($summary.total) PASS=$($summary.pass) FAIL=$($summary.fail)"
Write-Host "CSV=$csvPath"
Write-Host "JSON=$jsonPath"

if ($summary.fail -gt 0) {
    $results |
        Where-Object { $_.status -eq "FAIL" } |
        Select-Object id, category, message |
        Format-Table -AutoSize
    if ($FailOnError) {
        exit 1
    }
}
