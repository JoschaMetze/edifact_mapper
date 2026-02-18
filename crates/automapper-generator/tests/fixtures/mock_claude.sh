#!/bin/bash
# Mock claude CLI that reads stdin and returns canned condition generation response.
# Used in integration tests for the generate-conditions subcommand.

# Read stdin (prompt) and discard it
cat > /dev/null

# Return a canned JSON response
cat <<'EOF'
{
  "conditions": [
    {
      "id": "1",
      "implementation": null,
      "confidence": "high",
      "reasoning": "Requires external context: message splitting status",
      "is_external": true,
      "external_name": "message_splitting"
    },
    {
      "id": "2",
      "implementation": "if ctx.transaktion.marktlokationen.is_empty() {\n    ConditionResult::False\n} else {\n    ConditionResult::True\n}",
      "confidence": "high",
      "reasoning": "Simple field existence check on Marktlokation list",
      "is_external": false
    },
    {
      "id": "3",
      "implementation": "ConditionResult::Unknown",
      "confidence": "medium",
      "reasoning": "Interpretation of temporal condition uncertain",
      "is_external": false
    }
  ]
}
EOF
