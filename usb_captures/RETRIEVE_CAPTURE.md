# Retrieve USB Capture Data

The capture file was created but empty. The data is likely still in Chrome.

## Option 1: Re-export from Chrome Console

1. Switch back to the Chrome DevTools Console
2. Run this command:
```javascript
copy(JSON.stringify(window.hidocProtocolLog, null, 2))
```

3. Then in Terminal:
```bash
pbpaste > ~/Documents/GitHub/hidoc/usb_captures/hidoc_capture_$(date +%Y%m%d_%H%M%S).json
```

## Option 2: Check localStorage

The monitor auto-saves to localStorage. In Chrome Console, run:
```javascript
console.log(localStorage.getItem('hidocProtocolLog'))
copy(localStorage.getItem('hidocProtocolLog'))
```

Then paste to file as above.

## Option 3: Export log entries count

Check how many entries were captured:
```javascript
console.log('Captured entries:', window.hidocProtocolLog.length)
window.hidocProtocolLog
```

If you see entries, use Option 1 to export them.
