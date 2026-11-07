import type { FileSystemNode } from "@/models";

const API_HOST = location.hostname || "192.168.4.1";
const API_URI = `http://${API_HOST}/api/v1`;

export default class FernClient {
  static async readDir(path: string): Promise<FileSystemNode[]> {
    const uri = `${API_URI}/vfs/read-dir?path=${path}`;
    const data = await fetchJson(uri);

    return data.map((element: any): FileSystemNode => ({
      path: element.path,
      nodeType: element.node_type,
      size: 0,
      lastModified: new Date(0),
    }));
  }
}

async function fetchJson(uri: string) {
  const response = await fetch(uri);
  if (!response.ok) throw new Error(`fetch ${uri} failed: ${response.status}`);
  return response.json();
}
