import type { FileSystemNode } from "@/models";

const API_HOST = location.hostname || "192.168.4.1";
// const API_HOST = "localhost:8081";
const API_URI = `http://${API_HOST}/api/v1`;

export default class FernClient {
  static async readDir(path: string): Promise<FileSystemNode[]> {
    const uri = `${API_URI}/vfs/read-dir?path=${path}`;
    const response = await fetchJson(uri);

    return response.map((element: any): FileSystemNode => ({
      path: element.path,
      nodeType: element.node_type.toLowerCase(),
      size: element.size || 0,
      lastModified: new Date(element.last_modified || 0),
    }));
  }

  static async createDirectory(path: string): Promise<boolean> {
    const uri = `${API_URI}/vfs/create-dir?path=${path}`;
    const response = await fetchJson(uri);

    return response.status === "ok";
  }

  static async remove(path: string): Promise<boolean> {
    const uri = `${API_URI}/vfs/remove?path=${path}`;
    const response = await fetchJson(uri);

    return response.status === "ok";
  }
}

async function fetchJson(uri: string) {
  const response = await fetch(uri);
  if (!response.ok) throw new Error(`fetch ${uri} failed: ${response.status}`);
  return response.json();
}
