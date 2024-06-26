#!/bin/sh
echo "Generating client for Vehicle Management..." && \

rm -rf vehicle_management_service && \

openapi-generator generate -g rust -i vp-kuljetus-transport-management-specs/services/vehicle-management-services.yaml \
 -o vehicle_management_service --additional-properties=useSingleRequestParameter=true,packageName=vehicle-management-service \
 --global-property models,apis,supportingFiles,modelDocs=false,apiDocs=false && \

cd vehicle_management_service && rm .travis.yml git_push.sh README.md && \

echo "Client generated successfully!"