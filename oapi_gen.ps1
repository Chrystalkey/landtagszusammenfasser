# This script will generate the code for the collector and database from the OpenAPI spec
# It will download the openapi-generator-cli if it is not already present
# The generated code will be placed in the collector/oapicode and database/oapicode directories

echo "[OAPI-Gen] Generating Client and Server Code from Spec"
Set-Location -Path "ltzf-backend"
Invoke-Expression -Command .\oapigen.ps1
Set-Location -Path "../ltzf-collector"
Invoke-Expression -Command .\oapigen.ps1
Set-Location -Path "../ltzf-website"
Invoke-Expression -Command .\oapigen.ps1
Set-Location -Path ".."
echo "[OAPI-Gen] Done Generating Client and Server Code"