#!/bin/bash
# HiDoc P1 USB Device Enumeration Script
# Uses macOS ioreg to enumerate USB device details

set -e

echo "=================================================="
echo "HiDoc P1 USB Device Enumeration"
echo "=================================================="
echo ""

# Check if device is connected
if ! system_profiler SPUSBDataType | grep -q "HiDock"; then
    echo "ERROR: HiDoc P1 device not found!"
    echo "Please connect the device and try again."
    exit 1
fi

echo "✓ HiDoc P1 device detected"
echo ""

echo "=================================================="
echo "Audio Interface (Actions Semiconductor)"
echo "=================================================="
ioreg -p IOUSB -w0 -l | grep -A 30 "HiDock_P1" | head -35
echo ""

echo "=================================================="
echo "Control Interface (Solid State System)"
echo "=================================================="
ioreg -p IOUSB -w0 -l | grep -B 5 -A 30 "idVendor.*1395" | grep -A 30 "idProduct.*5d" | head -35
echo ""

echo "=================================================="
echo "USB Interfaces and Endpoints"
echo "=================================================="
ioreg -p IOUSB -w0 -l -r | grep -A 50 "HiDock" | grep -E "(bInterfaceNumber|bInterfaceClass|bEndpointAddress|bNumEndpoints)" | head -20
echo ""

echo "=================================================="
echo "Audio Device Information"
echo "=================================================="
system_profiler SPAudioDataType | grep -A 10 "HiDock" || echo "No audio device info found"
echo ""

echo "Done!"
