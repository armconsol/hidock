#!/bin/bash
# USB Packet Capture Script for HiDoc P1 Device
# Captures USB traffic for protocol reverse engineering

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CAPTURE_DIR="$SCRIPT_DIR/../usb_captures"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
CAPTURE_FILE="$CAPTURE_DIR/hidoc_p1_${TIMESTAMP}.pcapng"

# HiDoc P1 device identifiers
AUDIO_VID="10d6"
AUDIO_PID="b00e"
CONTROL_VID="1395"
CONTROL_PID="005d"

echo "=================================================="
echo "HiDoc P1 USB Packet Capture Script"
echo "=================================================="
echo ""
echo "Device identifiers:"
echo "  Audio Interface:   VID=$AUDIO_VID PID=$AUDIO_PID"
echo "  Control Interface: VID=$CONTROL_VID PID=$CONTROL_PID"
echo ""
echo "Capture file: $CAPTURE_FILE"
echo ""

# Create capture directory
mkdir -p "$CAPTURE_DIR"

# Check for Wireshark/tshark
if ! command -v tshark &> /dev/null; then
    echo "ERROR: tshark not found. Install with: brew install wireshark"
    exit 1
fi

# Check if device is connected
echo "Checking for HiDoc P1 device..."
if system_profiler SPUSBDataType | grep -q "HiDock"; then
    echo "✓ HiDoc P1 device detected"
else
    echo "✗ HiDoc P1 device NOT detected"
    echo "  Please connect the device and try again"
    exit 1
fi

echo ""
echo "=================================================="
echo "IMPORTANT: Perform these operations while capturing:"
echo "=================================================="
echo ""
echo "1. DEVICE DETECTION"
echo "   - Device should already be connected"
echo "   - Unplug and replug to capture enumeration"
echo ""
echo "2. RECORDING"
echo "   - Press record button on HiDoc P1"
echo "   - Record for 5-10 seconds"
echo "   - Press stop"
echo ""
echo "3. PLAYBACK"
echo "   - Press play button"
echo "   - Listen to recorded audio"
echo "   - Press stop"
echo ""
echo "4. FILE OPERATIONS (if available via HiNotes app)"
echo "   - Sync files"
echo "   - Transfer recordings"
echo ""
echo "5. CONFIGURATION (if available)"
echo "   - Change any device settings"
echo ""
echo "=================================================="
echo ""
echo "Press ENTER to start capture (Ctrl-C to stop)..."
read

# Start tshark capture with USB filter
echo "Starting USB capture..."
echo "Filter: usb.idVendor==$AUDIO_VID or usb.idVendor==$CONTROL_VID"
echo ""

sudo tshark -i usbmon0 \
    -f "usb.idVendor == 0x$AUDIO_VID or usb.idVendor == 0x$CONTROL_VID" \
    -w "$CAPTURE_FILE" \
    -P \
    2>&1 | while IFS= read -r line; do
        echo "$line"
    done

echo ""
echo "=================================================="
echo "Capture Complete!"
echo "=================================================="
echo ""
echo "Capture file saved to:"
echo "  $CAPTURE_FILE"
echo ""
echo "To analyze with Wireshark:"
echo "  wireshark $CAPTURE_FILE"
echo ""
echo "To analyze with tshark:"
echo "  tshark -r $CAPTURE_FILE"
echo ""
echo "To export as text:"
echo "  tshark -r $CAPTURE_FILE > ${CAPTURE_FILE%.pcapng}.txt"
echo ""
