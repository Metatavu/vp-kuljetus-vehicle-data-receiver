#!/bin/sh
echo "Generating client for Vehicle Management..."

libninja gen --lang rust --examples false -o vehicle_management_service_client vehicle-management-service-client vp-kuljetus-transport-management-specs/specs/vehicle-data-receiver.yaml

cd vehicle_management_service_client && rm -rf .github

echo "Client generated successfully!"