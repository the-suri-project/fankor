#!/bin/bash

# Make script fail if any command fails
set -e

# Move to folder
cd ../ts-client

# Lint the client
yarn lint:fix

# Build the client
yarn build

# Publish the client
yarn publish