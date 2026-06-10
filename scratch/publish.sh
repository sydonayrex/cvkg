#!/bin/bash

set -e

CRATES=(
    "cvkg-runic-text"
    "cvkg-core"
    "cvkg-anim"
    "cvkg-themes"
    "cvkg-scene"
    "cvkg-vdom"
    "cvkg-layout"
    "cvkg-components"
    "cvkg-render-web"
    "cvkg-compositor"
    "cvkg-svg-serialize"
    "cvkg-svg-filters"
    "cvkg-render-gpu"
    "cvkg-macros"
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
    
    # Retry loop in case the registry index hasn't propagated the previously published dependency
    MAX_RETRIES=5
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
    
    # Sleep to allow crates.io index to update before publishing the next dependent crate
    echo "Published $CRATE successfully. Waiting 15s for index propagation..."
    sleep 15
done

echo "All crates published successfully!"
