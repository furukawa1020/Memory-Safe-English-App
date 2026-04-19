param(
    [string]$ProxyBaseUrl = "http://127.0.0.1:8070",
    [string]$AdminToken = "dev-proxy-admin-token"
)

$ErrorActionPreference = "Stop"

function Invoke-JsonRequest {
    param(
        [string]$Method,
        [string]$Url,
        [hashtable]$Headers = @{},
        [object]$Body = $null
    )

    $params = @{
        Method      = $Method
        Uri         = $Url
        Headers     = $Headers
        ContentType = "application/json"
    }

    if ($null -ne $Body) {
        $params.Body = ($Body | ConvertTo-Json -Depth 8)
    }

    Invoke-RestMethod @params
}

Write-Host "1. Checking proxy health..."
$health = Invoke-JsonRequest -Method "GET" -Url "$ProxyBaseUrl/health"
$health | ConvertTo-Json -Depth 5 | Write-Host

Write-Host "2. Registering a test user through the proxy..."
$register = Invoke-JsonRequest -Method "POST" -Url "$ProxyBaseUrl/api/auth/register" -Body @{
    email = "smoke@example.com"
    password = "correct horse battery staple"
    display_name = "Smoke Test"
}
$accessToken = $register.access_token

Write-Host "3. Listing seeded contents..."
$contents = Invoke-JsonRequest -Method "GET" -Url "$ProxyBaseUrl/api/contents"
$firstContentId = $contents.items[0].id
$contents | ConvertTo-Json -Depth 6 | Write-Host

Write-Host "4. Requesting chunk analysis for the first content..."
$chunkResult = Invoke-JsonRequest -Method "GET" -Url "$ProxyBaseUrl/api/contents/$firstContentId/chunks" -Headers @{
    Authorization = "Bearer $accessToken"
}
$chunkResult | ConvertTo-Json -Depth 6 | Write-Host

Write-Host "5. Checking proxy cache stats..."
$cacheStats = Invoke-JsonRequest -Method "GET" -Url "$ProxyBaseUrl/admin/cache" -Headers @{
    "X-Proxy-Admin-Token" = $AdminToken
}
$cacheStats | ConvertTo-Json -Depth 5 | Write-Host

Write-Host "Smoke test completed."
