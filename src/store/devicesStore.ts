import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { Device, BindDeviceRequest } from '../types/devices';

interface DevicesState {
  devices: Device[];
  loading: boolean;
  error: string | null;

  // Actions
  fetchDevices: () => Promise<void>;
  bindDevice: (request: BindDeviceRequest) => Promise<void>;
  unbindDevice: (deviceId: string) => Promise<void>;
  updateDeviceStatus: (deviceId: string, status: 'connected' | 'disconnected') => Promise<void>;
}

export const useDevicesStore = create<DevicesState>((set) => ({
  devices: [],
  loading: false,
  error: null,

  fetchDevices: async () => {
    set({ loading: true, error: null });
    try {
      const devices = await invoke<Device[]>('list_devices');
      set({ devices, loading: false });
    } catch (error) {
      set({ error: String(error), loading: false });
    }
  },

  bindDevice: async (request: BindDeviceRequest) => {
    set({ loading: true, error: null });
    try {
      const device = await invoke<Device>('bind_device', { request });
      set((state) => ({
        devices: [...state.devices, device],
        loading: false,
      }));
    } catch (error) {
      set({ error: String(error), loading: false });
      throw error;
    }
  },

  unbindDevice: async (deviceId: string) => {
    set({ loading: true, error: null });
    try {
      await invoke('unbind_device', { deviceId });
      set((state) => ({
        devices: state.devices.filter((d) => d.id !== deviceId),
        loading: false,
      }));
    } catch (error) {
      set({ error: String(error), loading: false });
      throw error;
    }
  },

  updateDeviceStatus: async (deviceId: string, status: 'connected' | 'disconnected') => {
    try {
      const updated = await invoke<Device>('update_device_status', { deviceId, status });
      set((state) => ({
        devices: state.devices.map((d) => (d.id === deviceId ? updated : d)),
      }));
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },
}));
