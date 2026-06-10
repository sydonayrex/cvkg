#!/bin/bash

set -e

# 1. Temporarily strip cvkg-components from cvkg-macros dev-dependencies
sed -i '/cvkg-components/d' cvkg-macros/Cargo.toml

echo "Publishing cvkg-macros first (to break the cycle)..."
MAX_RETRIES=5
RETRY_COUNT=0
SUCCESS=false
while [ $RETRY_COUNT -lt $MAX_RETRIES ]; do
    if cargo publish --allow-dirty -p "cvkg-macros"; then
        SUCCESS=true
        break
    else
        echo "Publish failed, retrying in 10 seconds... ($(($RETRY_COUNT + 1))/$MAX_RETRIES)"
        sleep 10
        RETRY_COUNT=$(($RETRY_COUNT + 1))
    fi
done

if [ "$SUCCESS" = false ]; then
    echo "Failed to publish cvkg-macros."
    git restore cvkg-macros/Cargo.toml
    exit 1
fi

echo "Published cvkg-macros successfully. Waiting 15s for index propagation..."
sleep 15

# Restore cvkg-macros Cargo.toml
git restore cvkg-macros/Cargo.toml

# 2. Publish the remaining crates
CRATES=(
    "cvkg-components"
    "cvkg-render-web"
    "cvkg-compositor"
    "cvkg-svg-serialize"
    "cvkg-svg-filters"
    "cvkg-render-gpu"
    "cvkg-render-native"
    "cvkg"
    "cvkg-physics"
    "cvkg-cli"
    "cvkg-webkit-server"
    "cvkg-flow"
)

for CRATE in "${CRATES[@]}"; do
    echo "======================================"
    echo "Publishing $CRATE..."
    echo "======================================"
    
    RETRY_COUNT=0
    SUCCESS=false
    
    while [ $RETRY_COUNT -lt $MAX_RETRIES ]; do
        if cargo publish --allow-dirty -p "$CRATE"; then
            SUCCESS=true
            break
        else
            echo "Publish failed, retrying in 10 seconds... ($(($RETRY_COUNT + 1))/$MAX_RETRIES)"
            sleep 10
            RETRY_COUNT=$(($RETRY_COUNT + 1))
        fi
    done
    
    if [ "$SUCCESS" = false ]; then
        echo "Failed to publish $CRATE after $MAX_RETRIES retries."
        exit 1
    fi
    
    echo "Published $CRATE successfully. Waiting 15s for index propagation..."
    sleep 15
done

echo "All remaining crates published successfully!"
