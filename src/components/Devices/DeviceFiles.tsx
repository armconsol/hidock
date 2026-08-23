import { useState, useEffect } from 'react';
import {
  Table,
  Button,
  Progress,
  Upload,
  Empty,
  Space,
  message,
  Badge,
  Popconfirm,
  Card,
  Tooltip,
  Checkbox,
} from 'antd';
import type { CheckboxChangeEvent } from 'antd/es/checkbox';
import {
  DownloadOutlined,
  SyncOutlined,
  UploadOutlined,
  FileOutlined,
  DeleteOutlined,
  ReloadOutlined,
} from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import { invoke } from '@tauri-apps/api/core';
import './DeviceFiles.css';

export interface DeviceFile {
  id: string;
  name: string;
  size: number;
  duration: number | null;
  created_at: string;
  synced: boolean;
}

export interface DeviceInfo {
  id: string;
  name: string;
  storage_used: number;
  storage_total: number;
  last_sync: string | null;
}

interface DeviceFilesProps {
  deviceId: string;
}

interface FileTransfer {
  fileId: string;
  progress: number;
  status: 'pending' | 'downloading' | 'uploading' | 'completed' | 'error';
  error?: string;
}

export function DeviceFiles({ deviceId }: DeviceFilesProps) {
  const [files, setFiles] = useState<DeviceFile[]>([]);
  const [deviceInfo, setDeviceInfo] = useState<DeviceInfo | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedFileIds, setSelectedFileIds] = useState<string[]>([]);
  const [transfers, setTransfers] = useState<Map<string, FileTransfer>>(new Map());
  const [syncingAll, setSyncingAll] = useState(false);

  useEffect(() => {
    fetchDeviceInfo();
    fetchFiles();
  }, [deviceId]);

  const fetchDeviceInfo = async () => {
    try {
      const info = await invoke<DeviceInfo>('get_device_info', { deviceId });
      setDeviceInfo(info);
    } catch (err) {
      console.error('Failed to fetch device info:', err);
    }
  };

  const fetchFiles = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<DeviceFile[]>('list_device_files', { deviceId });
      setFiles(result);
    } catch (err) {
      setError(String(err));
      message.error('Failed to load device files');
    } finally {
      setLoading(false);
    }
  };

  const downloadFile = async (fileId: string, fileName: string) => {
    setTransfers((prev) => {
      const next = new Map(prev);
      next.set(fileId, { fileId, progress: 0, status: 'downloading' });
      return next;
    });

    try {
      // Simulate progress updates (in real implementation, this would come from backend)
      const progressInterval = setInterval(() => {
        setTransfers((prev) => {
          const next = new Map(prev);
          const current = next.get(fileId);
          if (current && current.progress < 90) {
            next.set(fileId, { ...current, progress: current.progress + 10 });
          }
          return next;
        });
      }, 200);

      await invoke('download_device_file', { deviceId, fileId, fileName });

      clearInterval(progressInterval);
      setTransfers((prev) => {
        const next = new Map(prev);
        next.set(fileId, { fileId, progress: 100, status: 'completed' });
        return next;
      });

      // Update file synced status
      setFiles((prev) =>
        prev.map((file) => (file.id === fileId ? { ...file, synced: true } : file))
      );

      message.success(`Downloaded ${fileName}`);

      // Clear transfer status after 2 seconds
      setTimeout(() => {
        setTransfers((prev) => {
          const next = new Map(prev);
          next.delete(fileId);
          return next;
        });
      }, 2000);
    } catch (err) {
      setTransfers((prev) => {
        const next = new Map(prev);
        next.set(fileId, {
          fileId,
          progress: 0,
          status: 'error',
          error: String(err),
        });
        return next;
      });
      message.error(`Failed to download ${fileName}`);
    }
  };

  const syncAllFiles = async () => {
    const unsyncedFiles = files.filter((f) => !f.synced);
    if (unsyncedFiles.length === 0) {
      message.info('All files are already synced');
      return;
    }

    setSyncingAll(true);
    let successCount = 0;
    let errorCount = 0;

    for (const file of unsyncedFiles) {
      try {
        await downloadFile(file.id, file.name);
        successCount++;
      } catch {
        errorCount++;
      }
    }

    setSyncingAll(false);

    if (errorCount === 0) {
      message.success(`Successfully synced ${successCount} files`);
    } else {
      message.warning(
        `Synced ${successCount} files, ${errorCount} failed`
      );
    }

    await fetchDeviceInfo();
  };

  const handleFileUpload = async (file: File) => {
    const uploadId = `upload-${Date.now()}`;

    setTransfers((prev) => {
      const next = new Map(prev);
      next.set(uploadId, { fileId: uploadId, progress: 0, status: 'uploading' });
      return next;
    });

    try {
      const progressInterval = setInterval(() => {
        setTransfers((prev) => {
          const next = new Map(prev);
          const current = next.get(uploadId);
          if (current && current.progress < 90) {
            next.set(uploadId, { ...current, progress: current.progress + 15 });
          }
          return next;
        });
      }, 300);

      await invoke('upload_file_to_device', {
        deviceId,
        fileName: file.name,
        fileData: await file.arrayBuffer(),
      });

      clearInterval(progressInterval);
      setTransfers((prev) => {
        const next = new Map(prev);
        next.set(uploadId, { fileId: uploadId, progress: 100, status: 'completed' });
        return next;
      });

      message.success(`Uploaded ${file.name}`);

      setTimeout(() => {
        setTransfers((prev) => {
          const next = new Map(prev);
          next.delete(uploadId);
          return next;
        });
      }, 2000);

      await fetchFiles();
      await fetchDeviceInfo();
    } catch (err) {
      setTransfers((prev) => {
        const next = new Map(prev);
        next.set(uploadId, {
          fileId: uploadId,
          progress: 0,
          status: 'error',
          error: String(err),
        });
        return next;
      });
      message.error(`Failed to upload ${file.name}`);
    }

    return false; // Prevent default upload behavior
  };

  const deleteFile = async (fileId: string, fileName: string) => {
    try {
      await invoke('delete_device_file', { deviceId, fileId });
      setFiles((prev) => prev.filter((f) => f.id !== fileId));
      message.success(`Deleted ${fileName}`);
      await fetchDeviceInfo();
    } catch (err) {
      message.error(`Failed to delete ${fileName}`);
    }
  };

  const retryTransfer = (fileId: string, fileName: string) => {
    downloadFile(fileId, fileName);
  };

  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
  };

  const formatDuration = (seconds: number | null): string => {
    if (seconds === null) return '-';
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  const formatDate = (dateStr: string): string => {
    const date = new Date(dateStr);
    return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
  };

  const handleSelectAll = (e: CheckboxChangeEvent) => {
    if (e.target.checked) {
      setSelectedFileIds(files.map((f) => f.id));
    } else {
      setSelectedFileIds([]);
    }
  };

  const handleSelectFile = (fileId: string, e: CheckboxChangeEvent) => {
    if (e.target.checked) {
      setSelectedFileIds((prev) => [...prev, fileId]);
    } else {
      setSelectedFileIds((prev) => prev.filter((id) => id !== fileId));
    }
  };

  const downloadSelectedFiles = async () => {
    if (selectedFileIds.length === 0) {
      message.info('No files selected');
      return;
    }

    const filesToDownload = files.filter((f) => selectedFileIds.includes(f.id));
    for (const file of filesToDownload) {
      await downloadFile(file.id, file.name);
    }

    setSelectedFileIds([]);
  };

  const columns: ColumnsType<DeviceFile> = [
    {
      title: (
        <Checkbox
          checked={selectedFileIds.length === files.length && files.length > 0}
          indeterminate={
            selectedFileIds.length > 0 && selectedFileIds.length < files.length
          }
          onChange={handleSelectAll}
        />
      ),
      dataIndex: 'select',
      width: 50,
      render: (_, record) => (
        <Checkbox
          checked={selectedFileIds.includes(record.id)}
          onChange={(e) => handleSelectFile(record.id, e)}
        />
      ),
    },
    {
      title: 'Name',
      dataIndex: 'name',
      sorter: (a, b) => a.name.localeCompare(b.name),
      render: (name) => (
        <Space>
          <FileOutlined />
          <span>{name}</span>
        </Space>
      ),
    },
    {
      title: 'Size',
      dataIndex: 'size',
      sorter: (a, b) => a.size - b.size,
      render: (size) => formatFileSize(size),
      width: 120,
    },
    {
      title: 'Duration',
      dataIndex: 'duration',
      sorter: (a, b) => (a.duration || 0) - (b.duration || 0),
      render: (duration) => formatDuration(duration),
      width: 100,
    },
    {
      title: 'Date',
      dataIndex: 'created_at',
      sorter: (a, b) =>
        new Date(a.created_at).getTime() - new Date(b.created_at).getTime(),
      render: (date) => formatDate(date),
      width: 180,
    },
    {
      title: 'Status',
      dataIndex: 'synced',
      filters: [
        { text: 'Synced', value: true },
        { text: 'Not Synced', value: false },
      ],
      onFilter: (value, record) => record.synced === value,
      render: (synced) =>
        synced ? (
          <Badge status="success" text="Synced" />
        ) : (
          <Badge status="warning" text="Not Synced" />
        ),
      width: 120,
    },
    {
      title: 'Actions',
      dataIndex: 'actions',
      width: 150,
      render: (_, record) => {
        const transfer = transfers.get(record.id);

        if (transfer) {
          if (transfer.status === 'error') {
            return (
              <Button
                type="text"
                size="small"
                icon={<ReloadOutlined />}
                onClick={() => retryTransfer(record.id, record.name)}
              >
                Retry
              </Button>
            );
          }

          return (
            <Progress
              percent={transfer.progress}
              size="small"
              status={transfer.status === 'completed' ? 'success' : undefined}
            />
          );
        }

        return (
          <Space>
            <Tooltip title="Download file">
              <Button
                type="text"
                size="small"
                icon={<DownloadOutlined />}
                onClick={() => downloadFile(record.id, record.name)}
              />
            </Tooltip>
            <Popconfirm
              title="Delete File"
              description={`Are you sure you want to delete ${record.name}?`}
              onConfirm={() => deleteFile(record.id, record.name)}
            >
              <Tooltip title="Delete file">
                <Button
                  type="text"
                  size="small"
                  danger
                  icon={<DeleteOutlined />}
                />
              </Tooltip>
            </Popconfirm>
          </Space>
        );
      },
    },
  ];

  const unsyncedCount = files.filter((f) => !f.synced).length;
  const storagePercent = deviceInfo
    ? Math.round((deviceInfo.storage_used / deviceInfo.storage_total) * 100)
    : 0;

  return (
    <div className="device-files-container">
      {deviceInfo && (
        <Card className="device-info-card" bordered>
          <div className="device-info-header">
            <div className="device-info-section">
              <h3>{deviceInfo.name}</h3>
              <p className="device-info-text">Device ID: {deviceInfo.id}</p>
            </div>
            <div className="device-info-section">
              <p className="device-info-label">Storage</p>
              <Progress
                percent={storagePercent}
                style={{ width: 200 }}
                status={storagePercent > 90 ? 'exception' : undefined}
              />
              <p className="device-info-text">
                {formatFileSize(deviceInfo.storage_used)} /{' '}
                {formatFileSize(deviceInfo.storage_total)}
              </p>
            </div>
            <div className="device-info-section">
              <p className="device-info-label">Last Sync</p>
              <p className="device-info-text">
                {deviceInfo.last_sync
                  ? formatDate(deviceInfo.last_sync)
                  : 'Never'}
              </p>
            </div>
          </div>
        </Card>
      )}

      <Card className="device-files-card" bordered>
        <div className="device-files-header">
          <Space>
            <Button
              type="primary"
              icon={<SyncOutlined />}
              onClick={syncAllFiles}
              loading={syncingAll}
              disabled={unsyncedCount === 0}
            >
              Sync All {unsyncedCount > 0 && `(${unsyncedCount})`}
            </Button>
            <Button
              icon={<DownloadOutlined />}
              onClick={downloadSelectedFiles}
              disabled={selectedFileIds.length === 0}
            >
              Download Selected ({selectedFileIds.length})
            </Button>
            <Button icon={<ReloadOutlined />} onClick={fetchFiles} loading={loading}>
              Refresh
            </Button>
          </Space>

          <Upload.Dragger
            multiple
            accept="audio/*"
            beforeUpload={handleFileUpload}
            showUploadList={false}
          >
            <div className="upload-drag-area">
              <UploadOutlined style={{ fontSize: 32, color: 'var(--color-primary-6)' }} />
              <p className="upload-drag-text">
                Click or drag audio files here to upload
              </p>
            </div>
          </Upload.Dragger>
        </div>

        {error ? (
          <Empty
            description={`Error loading files: ${error}`}
            image={<FileOutlined style={{ fontSize: 64, color: '#c9cdd4' }} />}
          />
        ) : files.length === 0 ? (
          <Empty
            description="No files on this device. Upload files to get started."
            image={<FileOutlined style={{ fontSize: 64, color: '#c9cdd4' }} />}
          />
        ) : (
          <Table
            columns={columns}
            dataSource={files}
            loading={loading}
            pagination={{
              pageSize: 20,
              showTotal: (total, range) => `${range[0]}-${range[1]} of ${total} items`,
              showQuickJumper: true,
            }}
            rowKey="id"
            className="device-files-table"
          />
        )}

        {Array.from(transfers.values()).some(
          (t) => t.status === 'uploading' || t.status === 'downloading'
        ) && (
          <div className="transfer-status-bar">
            <Space>
              {Array.from(transfers.values())
                .filter(
                  (t) => t.status === 'uploading' || t.status === 'downloading'
                )
                .map((transfer) => (
                  <div key={transfer.fileId} className="transfer-item">
                    <span>
                      {transfer.status === 'uploading' ? 'Uploading' : 'Downloading'}
                    </span>
                    <Progress
                      percent={transfer.progress}
                      size="small"
                      style={{ width: 150 }}
                    />
                  </div>
                ))}
            </Space>
          </div>
        )}
      </Card>
    </div>
  );
}
