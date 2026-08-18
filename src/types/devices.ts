export interface Device {
  id: string;
  name: string;
  status: 'connected' | 'disconnected';
  last_sync: string | null;
  created_at: string;
}

export interface BindDeviceRequest {
  id: string;
  name: string;
}
