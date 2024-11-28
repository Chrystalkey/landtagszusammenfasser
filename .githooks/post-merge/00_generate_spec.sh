mkdir oai-generator
cd oai-generator
curl https://raw.githubusercontent.com/OpenAPITools/openapi-generator/master/bin/utils/openapi-generator-cli.sh > ~/bin/openapitools/openapi-generator-cli
chmod u+x ~/bin/openapitools/openapi-generator-cli
export PATH=$PATH:$(pwd)
cd ..

openapi-generator-cli generate -g python -i $(pwd)/docs/specs/openapi.yml -o $(pwd)/collector/oaiclient

