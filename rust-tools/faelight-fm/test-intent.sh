#!/bin/bash
# Check if INTENT dir exists
if [ -d ~/0-core/INTENT/future ]; then
    echo "INTENT dir exists"
    ls ~/0-core/INTENT/future/074* 2>/dev/null || echo "074 not found"
else
    echo "INTENT dir NOT found"
fi
