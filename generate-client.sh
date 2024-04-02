#!/bin/sh
echo "Generating client for Vehicle Management..."

libninja gen --lang rust --examples false -o vehicle-management-service vehicle-management-service vp-kuljetus-transport-management-specs/specs/vehicle-data-receiver.yaml

cd vehicle-management-service && rm -rf .github

echo "Client generated successfully!"