# VHS tape file for SARA main demo
# Run with: vhs demo.tape

# Output configuration
Output ../generated/demo.gif

# Include common settings
Source common.tape

# Setup: navigate to examples directory (hidden)
Hide
Type@0 "cd ../.. && clear"
Enter
Show

# Navigate to smart-home example
Type "cd examples/smart-home"
Enter
Sleep 500ms

# Parse all documents
Type "sara parse"
Enter
Sleep 2s

# Validate
Type "sara validate"
Enter
Sleep 2s

# Query with upstream traceability
Type "sara query SWREQ-MQTTCLIENT --upstream"
Enter
Sleep 3s

# Query with downstream traceability
Type "sara report coverage"
Enter
Sleep 3s