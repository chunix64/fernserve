import type { FileSystemNode } from "@/models";

export default class FernClient {
  static async readDir(path: string): Promise<FileSystemNode[]> {
    const uri = "http://localhost:8081/";
    const data = await fetchJson(uri);

    return data.map((element: any): FileSystemNode => ({
      path: element.path,
      nodeType: element.nodeType,
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
