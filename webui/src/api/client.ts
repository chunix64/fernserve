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
      nodeType: element.metadata.node_type.toLowerCase(),
      size: element.metadata.len || 0,
      lastModified: new Date(element.metadata.last_modified || 0),
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

  static async removeAll(path: string): Promise<boolean> {
    const uri = `${API_URI}/vfs/remove-all?path=${path}`;
    const response = await fetchJson(uri);

    return response.status === "ok";
  }

  static async uploadFile(path: string, file: File): Promise<boolean> {
    const uri = `${API_URI}/vfs/upload-file?path=${path}`;
    const response = await fetchJsonWithFile(uri, file);

    return response.status === "ok";
  }

  static async downloadFile(path: string) {
    const uri = `${API_URI}/vfs/download-file?path=${path}`;
    const a = document.createElement("a");
    a.href = uri;
    a.download = path.split("/").pop() ?? "unknow_ferserve_file";
    document.body.appendChild(a);
    a.click();
    a.remove();
  }
}

// Helpers
async function fetchJson(uri: string) {
  const response = await fetch(uri);
  if (!response.ok) throw new Error(`fetch ${uri} failed: ${response.status}`);
  return response.json();
}

async function fetchJsonWithFile(uri: string, file: File) {
  const response = await fetch(uri, {
    method: "POST",
    body: file,
  });
  if (!response.ok) throw new Error(`fetch ${uri} failed: ${response.status}`);
  return response.json();
}
