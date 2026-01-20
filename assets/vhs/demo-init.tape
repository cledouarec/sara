# VHS tape file for SARA init command demo
# Run with: vhs demo-init.tape

# Output configuration
Output ../generated/demo-init.gif

# Include common settings
Source common.tape

# Setup: create the markdown file (hidden)
Hide
Type@0 "echo -e '# Customer Portal Solution\n\nThis solution provides a self-service portal for customers.' > customer-portal.md && clear"
Enter
Show

# Start the demo
Type "# Display an existing markdown file"
Enter
Sleep 500ms

Type "cat customer-portal.md"
Enter
Sleep 2s

Type "# Initialize metadata with sara init"
Enter
Sleep 500ms

Type "sara init customer-portal.md"
Enter
Sleep 1s

# Select item type: Solution (first option, just press Enter)
Enter
Sleep 500ms

# Enter item name (use extracted title as default)
Type "Customer Portal"
Enter
Sleep 500ms

# Enter identifier (accept suggested default)
Enter
Sleep 500ms

# Enter description
Type "Web-based customer self-service portal"
Enter
Sleep 500ms

# Confirm creation
Enter
Sleep 2s

Type "# View the generated frontmatter"
Enter
Sleep 500ms

Type "cat customer-portal.md"
Enter
Sleep 3s

# Cleanup (hidden)
Hide
Type "rm customer-portal.md"
Enter
Show
