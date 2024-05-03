#!/bin/sh
echo "Generating client for Vehicle Management..." && \

rm -rf vehicle_management_service && \

openapi-generator generate -g rust -i vp-kuljetus-transport-management-specs/specs/vehicle-data-receiver.yaml \
 -o vehicle_management_service --additional-properties=supportAsync=false,useSingleRequestParameter=true,packageName=vehicle-management-service \
 --global-property models,apis,supportingFiles,modelDocs=false,apiDocs=false && \

cd vehicle_management_service && rm .travis.yml git_push.sh README.md && \

echo "Client generated successfully!"