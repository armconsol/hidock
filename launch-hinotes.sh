#!/bin/bash
# Launch script for HiNotes Desktop with OAuth credentials
#
# Usage:
#   1. Edit this file and add your GOOGLE_CLIENT_ID
#   2. chmod +x launch-hinotes.sh
#   3. ./launch-hinotes.sh

# OAuth Configuration
export GOOGLE_CLIENT_ID="your-client-id.apps.googleusercontent.com"
export GOOGLE_CLIENT_SECRET=""  # Optional

# Apple Sign In (optional)
export APPLE_CLIENT_ID=""
export APPLE_TEAM_ID=""
export APPLE_KEY_ID=""

# API Configuration (optional, defaults to production)
# export HINOTES_API_BASE="https://hinotes.hidock.com/v1"

# Launch the application
open -a "HiNotes Desktop"
