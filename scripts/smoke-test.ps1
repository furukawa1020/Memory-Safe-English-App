param(
    [string]$ProxyBaseUrl = "http://127.0.0.1:8070",
    [string]$AdminToken = "dev-proxy-admin-token",
    [string]$Email = "smoke@example.com",
    [string]$Password = "correct horse battery staple",
    [string]$DisplayName = "Smoke Test"
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

function Get-AuthTokens {
    param(
        [string]$BaseUrl,
        [string]$EmailAddress,
        [string]$PlaintextPassword,
        [string]$Name
    )

    try {
        $registerResponse = Invoke-JsonRequest -Method "POST" -Url "$BaseUrl/auth/register" -Body @{
            email            = $EmailAddress
            password         = $PlaintextPassword
            display_name     = $Name
            agreed_to_terms  = $true
        }

        return $registerResponse.tokens
    } catch {
        Write-Host "Registration did not succeed. Falling back to login..."
    }

    $loginResponse = Invoke-JsonRequest -Method "POST" -Url "$BaseUrl/auth/login" -Body @{
        email    = $EmailAddress
        password = $PlaintextPassword
    }

    return $loginResponse.tokens
}

Write-Host "1. Checking proxy readiness..."
$readiness = Invoke-JsonRequest -Method "GET" -Url "$ProxyBaseUrl/ready"
$readiness | ConvertTo-Json -Depth 8 | Write-Host

Write-Host "2. Registering or logging in the smoke-test user..."
$tokens = Get-AuthTokens -BaseUrl $ProxyBaseUrl -EmailAddress $Email -PlaintextPassword $Password -Name $DisplayName
$accessToken = $tokens.access_token
if (-not $accessToken) {
    throw "Smoke test could not obtain an access token."
}

$authHeaders = @{
    Authorization = "Bearer $accessToken"
}

Write-Host "3. Fetching the current user..."
$me = Invoke-JsonRequest -Method "GET" -Url "$ProxyBaseUrl/me" -Headers $authHeaders
$me | ConvertTo-Json -Depth 6 | Write-Host

Write-Host "4. Listing seeded contents..."
$contents = Invoke-JsonRequest -Method "GET" -Url "$ProxyBaseUrl/contents" -Headers $authHeaders
if (-not $contents.items -or $contents.items.Count -eq 0) {
    throw "Smoke test expected at least one seeded content item."
}
$firstContentId = $contents.items[0].content_id
$contents | ConvertTo-Json -Depth 8 | Write-Host

Write-Host "5. Requesting chunk analysis for the first content..."
$chunkResult = Invoke-JsonRequest -Method "GET" -Url "$ProxyBaseUrl/contents/$firstContentId/chunks" -Headers $authHeaders
$chunkResult | ConvertTo-Json -Depth 8 | Write-Host

Write-Host "6. Requesting skeleton analysis for the first content..."
$skeletonResult = Invoke-JsonRequest -Method "GET" -Url "$ProxyBaseUrl/contents/$firstContentId/skeleton" -Headers $authHeaders
$skeletonResult | ConvertTo-Json -Depth 8 | Write-Host

Write-Host "7. Checking proxy cache stats..."
$cacheStats = Invoke-JsonRequest -Method "GET" -Url "$ProxyBaseUrl/admin/cache" -Headers @{
    "X-Proxy-Admin-Token" = $AdminToken
}
$cacheStats | ConvertTo-Json -Depth 8 | Write-Host

Write-Host "Smoke test completed successfully."
