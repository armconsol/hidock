// HiDoc P1 WebUSB Protocol Monitor
// Paste this in Chrome DevTools Console BEFORE connecting device
// Or refresh page after pasting to capture from start

(function() {
  'use strict';

  // Storage for captured protocol data
  window.hidocProtocolLog = [];
  let logIndex = 0;

  function logEntry(type, data) {
    const entry = {
      index: logIndex++,
      timestamp: Date.now(),
      time: new Date().toISOString(),
      type: type,
      ...data
    };
    window.hidocProtocolLog.push(entry);
    return entry;
  }

  // Hook USB device request
  const originalRequestDevice = navigator.usb.requestDevice.bind(navigator.usb);
  navigator.usb.requestDevice = async function(...args) {
    console.log('%c[USB] requestDevice called', 'color: blue; font-weight: bold', args);
    logEntry('requestDevice', { args });
    const device = await originalRequestDevice(...args);
    console.log('%c[USB] Device granted:', 'color: green; font-weight: bold', device);
    logEntry('deviceGranted', {
      vendorId: '0x' + device.vendorId.toString(16),
      productId: '0x' + device.productId.toString(16),
      manufacturer: device.manufacturerName,
      product: device.productName,
      serial: device.serialNumber
    });
    return device;
  };

  // Hook device.open()
  const originalOpen = USBDevice.prototype.open;
  USBDevice.prototype.open = async function() {
    console.log('%c[USB] open()', 'color: purple; font-weight: bold');
    logEntry('open', { device: this.productName });
    return originalOpen.call(this);
  };

  // Hook device.close()
  const originalClose = USBDevice.prototype.close;
  USBDevice.prototype.close = async function() {
    console.log('%c[USB] close()', 'color: purple; font-weight: bold');
    logEntry('close', { device: this.productName });
    return originalClose.call(this);
  };

  // Hook device.selectConfiguration()
  const originalSelectConfiguration = USBDevice.prototype.selectConfiguration;
  USBDevice.prototype.selectConfiguration = async function(configurationValue) {
    console.log('%c[USB] selectConfiguration:', 'color: orange; font-weight: bold', configurationValue);
    logEntry('selectConfiguration', { configurationValue });
    return originalSelectConfiguration.call(this, configurationValue);
  };

  // Hook device.claimInterface()
  const originalClaimInterface = USBDevice.prototype.claimInterface;
  USBDevice.prototype.claimInterface = async function(interfaceNumber) {
    console.log('%c[USB] claimInterface:', 'color: orange; font-weight: bold', interfaceNumber);
    logEntry('claimInterface', { interfaceNumber });
    return originalClaimInterface.call(this, interfaceNumber);
  };

  // Hook controlTransferIn
  const originalControlTransferIn = USBDevice.prototype.controlTransferIn;
  USBDevice.prototype.controlTransferIn = async function(setup, length) {
    console.log('%c[USB] controlTransferIn:', 'color: cyan; font-weight: bold', setup, 'length:', length);
    const result = await originalControlTransferIn.call(this, setup, length);
    const data = new Uint8Array(result.data.buffer);
    console.log('%c  ← Response:', 'color: green', {
      status: result.status,
      bytesRead: data.length,
      data: Array.from(data),
      hex: Array.from(data).map(b => b.toString(16).padStart(2, '0')).join(' ')
    });
    logEntry('controlTransferIn', {
      setup,
      requestLength: length,
      status: result.status,
      bytesRead: data.length,
      data: Array.from(data),
      hex: Array.from(data).map(b => b.toString(16).padStart(2, '0')).join(' ')
    });
    return result;
  };

  // Hook controlTransferOut
  const originalControlTransferOut = USBDevice.prototype.controlTransferOut;
  USBDevice.prototype.controlTransferOut = async function(setup, data) {
    const dataArray = new Uint8Array(data);
    console.log('%c[USB] controlTransferOut:', 'color: cyan; font-weight: bold', setup);
    console.log('%c  → Data:', 'color: blue', {
      bytes: dataArray.length,
      data: Array.from(dataArray),
      hex: Array.from(dataArray).map(b => b.toString(16).padStart(2, '0')).join(' ')
    });
    logEntry('controlTransferOut', {
      setup,
      dataLength: dataArray.length,
      data: Array.from(dataArray),
      hex: Array.from(dataArray).map(b => b.toString(16).padStart(2, '0')).join(' ')
    });
    return originalControlTransferOut.call(this, setup, data);
  };

  // Hook transferIn
  const originalTransferIn = USBDevice.prototype.transferIn;
  USBDevice.prototype.transferIn = async function(endpointNumber, length) {
    console.log('%c[USB] transferIn:', 'color: magenta; font-weight: bold',
      'endpoint:', endpointNumber, 'length:', length);
    const result = await originalTransferIn.call(this, endpointNumber, length);
    const data = new Uint8Array(result.data.buffer);
    console.log('%c  ← Received:', 'color: green', {
      status: result.status,
      bytesRead: data.length,
      data: Array.from(data).slice(0, 64), // First 64 bytes
      hex: Array.from(data).slice(0, 64).map(b => b.toString(16).padStart(2, '0')).join(' ')
    });
    logEntry('transferIn', {
      endpoint: endpointNumber,
      requestLength: length,
      status: result.status,
      bytesRead: data.length,
      data: Array.from(data).slice(0, 64), // Save first 64 bytes
      hex: Array.from(data).slice(0, 64).map(b => b.toString(16).padStart(2, '0')).join(' ')
    });
    return result;
  };

  // Hook transferOut
  const originalTransferOut = USBDevice.prototype.transferOut;
  USBDevice.prototype.transferOut = async function(endpointNumber, data) {
    const dataArray = new Uint8Array(data);
    console.log('%c[USB] transferOut:', 'color: magenta; font-weight: bold',
      'endpoint:', endpointNumber);
    console.log('%c  → Data:', 'color: blue', {
      bytes: dataArray.length,
      data: Array.from(dataArray).slice(0, 64), // First 64 bytes
      hex: Array.from(dataArray).slice(0, 64).map(b => b.toString(16).padStart(2, '0')).join(' ')
    });
    logEntry('transferOut', {
      endpoint: endpointNumber,
      dataLength: dataArray.length,
      data: Array.from(dataArray).slice(0, 64),
      hex: Array.from(dataArray).slice(0, 64).map(b => b.toString(16).padStart(2, '0')).join(' ')
    });
    return originalTransferOut.call(this, endpointNumber, data);
  };

  console.log('%c✓ HiDoc P1 USB Monitor Installed!', 'color: green; font-size: 16px; font-weight: bold');
  console.log('%cNow refresh the page or connect your device', 'color: orange; font-size: 12px');
  console.log('%cProtocol log will be saved in: window.hidocProtocolLog', 'color: blue');
  console.log('%cTo export: copy(JSON.stringify(window.hidocProtocolLog, null, 2))', 'color: blue');

  // Helper function to export log
  window.exportHidocLog = function() {
    const json = JSON.stringify(window.hidocProtocolLog, null, 2);
    console.log('%cExporting ' + window.hidocProtocolLog.length + ' log entries...', 'color: green');
    copy(json);
    console.log('%c✓ Copied to clipboard! Paste into a text file.', 'color: green; font-weight: bold');
    return json;
  };

  // Auto-export on visibility change (when tab loses focus)
  document.addEventListener('visibilitychange', function() {
    if (document.hidden && window.hidocProtocolLog.length > 0) {
      console.log('%c[Auto-Save] Saving protocol log...', 'color: orange');
      localStorage.setItem('hidocProtocolLog', JSON.stringify(window.hidocProtocolLog));
      console.log('%c✓ Saved to localStorage', 'color: green');
    }
  });

  // Restore previous log if exists
  const savedLog = localStorage.getItem('hidocProtocolLog');
  if (savedLog) {
    try {
      const parsed = JSON.parse(savedLog);
      console.log('%c[Auto-Restore] Found previous log with ' + parsed.length + ' entries', 'color: orange');
      console.log('%cTo restore: window.hidocProtocolLog = ' + parsed.length + ' entries', 'color: blue');
    } catch(e) {}
  }

})();
