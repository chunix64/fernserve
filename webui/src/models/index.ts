export type FileSystemNode = {
  path: string;
  nodeType: string;
  size: number;
  lastModified: Date;
};

export type WifiEntry = {
  ssid: string;
  auth_method: string;
  rssi: number;
  channel: number;
};
