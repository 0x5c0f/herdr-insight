#!/bin/bash
# Wrapper script to open timeline with direction support
# Usage: open-timeline.sh [down|right]
DIRECTION="${1:-down}"
herdr plugin pane open --plugin herdr-insight --entrypoint timeline --direction "$DIRECTION"
