#!/bin/bash
# Issue #258: Storage collision checker for co-deployed contracts
# Checks for duplicate storage keys across all contracts

set -euo pipefail

echo "🔍 Checking for storage key collisions across contracts..."

CONTRACTS_DIR="contracts"
TEMP_FILE=$(mktemp)
COLLISION_FILE=$(mktemp)
EXIT_CODE=0

# Find all storage key definitions
echo "Scanning contracts for storage keys..."

# Common patterns for storage keys in Soroban contracts
grep -rh "const.*ADMIN" "$CONTRACTS_DIR" --include="*.rs" | sort > "$TEMP_FILE" || true
grep -rh "const.*INIT" "$CONTRACTS_DIR" --include="*.rs" | sort >> "$TEMP_FILE" || true
grep -rh "const.*STORAGE" "$CONTRACTS_DIR" --include="*.rs" | sort >> "$TEMP_FILE" || true
grep -rh "Symbol::new" "$CONTRACTS_DIR" --include="*.rs" | sort >> "$TEMP_FILE" || true

# Check for duplicate keys
echo ""
echo "Analyzing for collisions..."

# Extract just the key names and find duplicates
cat "$TEMP_FILE" | sort | uniq -d > "$COLLISION_FILE"

if [ -s "$COLLISION_FILE" ]; then
    echo ""
    echo "❌ Storage key collisions detected:"
    echo ""
    cat "$COLLISION_FILE"
    echo ""
    echo "Affected contracts:"
    
    while IFS= read -r key; do
        echo ""
        echo "Key: $key"
        grep -rn "$key" "$CONTRACTS_DIR" --include="*.rs" || true
    done < "$COLLISION_FILE"
    
    EXIT_CODE=1
else
    echo "✅ No storage key collisions detected"
fi

# Document key usage
echo ""
echo "📊 Storage key usage by contract:"
echo ""

for contract_dir in "$CONTRACTS_DIR"/*; do
    if [ -d "$contract_dir" ]; then
        contract_name=$(basename "$contract_dir")
        key_count=$(grep -rh "const.*:" "$contract_dir" --include="*.rs" 2>/dev/null | grep -i "key\|admin\|init\|storage" | wc -l || echo "0")
        
        if [ "$key_count" -gt 0 ]; then
            echo "$contract_name: $key_count storage keys"
        fi
    fi
done

# Cleanup
rm -f "$TEMP_FILE" "$COLLISION_FILE"

echo ""
if [ $EXIT_CODE -eq 0 ]; then
    echo "✅ Storage collision check passed"
else
    echo "❌ Storage collision check failed"
    echo ""
    echo "Remediation:"
    echo "- Use contract-specific prefixes (e.g., 'STAKING_ADMIN', 'TREASURY_ADMIN')"
    echo "- Review contracts/*/src/lib.rs for key definitions"
    echo "- Ensure each contract uses unique storage keys"
fi

exit $EXIT_CODE
