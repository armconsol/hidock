import { describe, it, expect, vi, beforeEach } from 'vitest';
import { useDevicesStore } from './devicesStore';
import { invoke } from '@tauri-apps/api/core';
import type { Device } from '../types/devices';

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('devicesStore', () => {
  const mockDevices: Device[] = [
    {
      id: 'HIDOC-001',
      name: 'My HiDoc P1',
      status: 'connected',
      last_sync: new Date().toISOString(),
      created_at: new Date().toISOString(),
    },
    {
      id: 'HIDOC-002',
      name: 'Office HiDoc',
      status: 'disconnected',
      last_sync: null,
      created_at: new Date().toISOString(),
    },
  ];

  beforeEach(() => {
    vi.clearAllMocks();
    useDevicesStore.setState({
      devices: [],
      loading: false,
      error: null,
    });
  });

  describe('fetchDevices', () => {
    it('fetches devices successfully', async () => {
      (invoke as ReturnType<typeof vi.fn>).mockResolvedValue(mockDevices);

      const store = useDevicesStore.getState();
      await store.fetchDevices();

      const state = useDevicesStore.getState();
      expect(state.devices).toEqual(mockDevices);
      expect(state.loading).toBe(false);
      expect(state.error).toBe(null);
      expect(invoke).toHaveBeenCalledWith('list_devices');
    });

    it('handles fetch error', async () => {
      const errorMessage = 'Failed to fetch devices';
      (invoke as ReturnType<typeof vi.fn>).mockRejectedValue(new Error(errorMessage));

      const store = useDevicesStore.getState();
      await store.fetchDevices();

      const state = useDevicesStore.getState();
      expect(state.devices).toEqual([]);
      expect(state.loading).toBe(false);
      expect(state.error).toBe(`Error: ${errorMessage}`);
    });

    it('sets loading state during fetch', async () => {
      (invoke as ReturnType<typeof vi.fn>).mockImplementation(
        () => new Promise((resolve) => setTimeout(() => resolve(mockDevices), 100))
      );

      const store = useDevicesStore.getState();
      const fetchPromise = store.fetchDevices();

      // Check loading state immediately
      expect(useDevicesStore.getState().loading).toBe(true);

      await fetchPromise;

      // Check loading state after completion
      expect(useDevicesStore.getState().loading).toBe(false);
    });
  });

  describe('bindDevice', () => {
    it('binds device successfully', async () => {
      const newDevice: Device = {
        id: 'HIDOC-003',
        name: 'New HiDoc',
        status: 'disconnected',
        last_sync: null,
        created_at: new Date().toISOString(),
      };

      (invoke as ReturnType<typeof vi.fn>).mockResolvedValue(newDevice);

      const store = useDevicesStore.getState();
      await store.bindDevice({ id: 'HIDOC-003', name: 'New HiDoc' });

      const state = useDevicesStore.getState();
      expect(state.devices).toContainEqual(newDevice);
      expect(state.loading).toBe(false);
      expect(state.error).toBe(null);
      expect(invoke).toHaveBeenCalledWith('bind_device', {
        request: { id: 'HIDOC-003', name: 'New HiDoc' },
      });
    });

    it('handles bind error', async () => {
      const errorMessage = 'Device already exists';
      (invoke as ReturnType<typeof vi.fn>).mockRejectedValue(new Error(errorMessage));

      const store = useDevicesStore.getState();

      await expect(
        store.bindDevice({ id: 'HIDOC-001', name: 'Duplicate' })
      ).rejects.toThrow();

      const state = useDevicesStore.getState();
      expect(state.loading).toBe(false);
      expect(state.error).toBe(`Error: ${errorMessage}`);
    });

    it('adds device to existing devices list', async () => {
      useDevicesStore.setState({ devices: [mockDevices[0]] });

      const newDevice: Device = {
        id: 'HIDOC-003',
        name: 'New HiDoc',
        status: 'disconnected',
        last_sync: null,
        created_at: new Date().toISOString(),
      };

      (invoke as ReturnType<typeof vi.fn>).mockResolvedValue(newDevice);

      const store = useDevicesStore.getState();
      await store.bindDevice({ id: 'HIDOC-003', name: 'New HiDoc' });

      const state = useDevicesStore.getState();
      expect(state.devices).toHaveLength(2);
      expect(state.devices).toContainEqual(mockDevices[0]);
      expect(state.devices).toContainEqual(newDevice);
    });
  });

  describe('unbindDevice', () => {
    beforeEach(() => {
      useDevicesStore.setState({ devices: mockDevices });
    });

    it('unbinds device successfully', async () => {
      (invoke as ReturnType<typeof vi.fn>).mockResolvedValue(undefined);

      const store = useDevicesStore.getState();
      await store.unbindDevice('HIDOC-001');

      const state = useDevicesStore.getState();
      expect(state.devices).toHaveLength(1);
      expect(state.devices[0].id).toBe('HIDOC-002');
      expect(state.loading).toBe(false);
      expect(state.error).toBe(null);
      expect(invoke).toHaveBeenCalledWith('unbind_device', { deviceId: 'HIDOC-001' });
    });

    it('handles unbind error', async () => {
      const errorMessage = 'Device not found';
      (invoke as ReturnType<typeof vi.fn>).mockRejectedValue(new Error(errorMessage));

      const store = useDevicesStore.getState();

      await expect(store.unbindDevice('HIDOC-999')).rejects.toThrow();

      const state = useDevicesStore.getState();
      expect(state.devices).toHaveLength(2); // Devices unchanged
      expect(state.loading).toBe(false);
      expect(state.error).toBe(`Error: ${errorMessage}`);
    });
  });

  describe('updateDeviceStatus', () => {
    beforeEach(() => {
      useDevicesStore.setState({ devices: mockDevices });
    });

    it('updates device status successfully', async () => {
      const updatedDevice: Device = {
        ...mockDevices[1],
        status: 'connected',
      };

      (invoke as ReturnType<typeof vi.fn>).mockResolvedValue(updatedDevice);

      const store = useDevicesStore.getState();
      await store.updateDeviceStatus('HIDOC-002', 'connected');

      const state = useDevicesStore.getState();
      const device = state.devices.find((d) => d.id === 'HIDOC-002');
      expect(device?.status).toBe('connected');
      expect(invoke).toHaveBeenCalledWith('update_device_status', {
        deviceId: 'HIDOC-002',
        status: 'connected',
      });
    });

    it('handles update status error', async () => {
      const errorMessage = 'Failed to update status';
      (invoke as ReturnType<typeof vi.fn>).mockRejectedValue(new Error(errorMessage));

      const store = useDevicesStore.getState();

      await expect(store.updateDeviceStatus('HIDOC-002', 'connected')).rejects.toThrow();

      const state = useDevicesStore.getState();
      expect(state.error).toBe(`Error: ${errorMessage}`);
    });

    it('updates only the specified device', async () => {
      const updatedDevice: Device = {
        ...mockDevices[1],
        status: 'connected',
      };

      (invoke as ReturnType<typeof vi.fn>).mockResolvedValue(updatedDevice);

      const store = useDevicesStore.getState();
      await store.updateDeviceStatus('HIDOC-002', 'connected');

      const state = useDevicesStore.getState();
      expect(state.devices[0]).toEqual(mockDevices[0]); // Unchanged
      expect(state.devices[1].status).toBe('connected'); // Updated
    });
  });
});
