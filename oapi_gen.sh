#/bin/sh
# This script will generate the code for the collector and database from the OpenAPI spec
# It will download the openapi-generator-cli if it is not already present
# The generated code will be placed in the collector/oapicode and database/oapicode directories

echo "[OAPI-Gen] Generating Client and Server Code from Spec"
cd "ltzf-backend"
sh oapigen.sh
cd "../ltzf-collector"
sh oapigen.sh
cd "../ltzf-website"
sh oapigen.sh
cd ..
echo "[OAPI-Gen] Done Generating Client and Server Code"