import { UPLOAD_CHUNK_SIZE, UPLOAD_TRY_ATTEMPT } from "@/configs";
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

    return response.status === 200;
  }

  static async remove(path: string): Promise<boolean> {
    const uri = `${API_URI}/vfs/remove?path=${path}`;
    const response = await fetchJson(uri);

    return response.status === 200;
  }

  static async removeAll(path: string): Promise<boolean> {
    const uri = `${API_URI}/vfs/remove-all?path=${path}`;
    const response = await fetchJson(uri);

    return response.status === 200;
  }

  static async uploadFile(
    path: string,
    file: File,
    onProgress?: (uploadedBytes: number) => void,
    signal?: AbortSignal,
  ): Promise<boolean> {
    const totalChunks = Math.max(1, Math.ceil(file.size / UPLOAD_CHUNK_SIZE));

    for (let chunkIndex = 0; chunkIndex < totalChunks; chunkIndex++) {
      const start = chunkIndex * UPLOAD_CHUNK_SIZE;
      const end = Math.min(start + UPLOAD_CHUNK_SIZE, file.size);
      const uri = `${API_URI}/vfs/upload-file` +
        `?path=${encodeURIComponent(path)}` +
        `&chunk_index=${chunkIndex}&total_chunks=${totalChunks}`;

      await fetchJsonWithChunk(
        uri,
        file.slice(start, end),
        UPLOAD_TRY_ATTEMPT,
        signal,
      );

      onProgress?.(end);
    }

    return true;
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

function sleep(ms: number, signal?: AbortSignal): Promise<void> {
  return new Promise((resolve, reject) => {
    if (signal?.aborted) return reject(signal.reason);

    const onAbort = () => {
      clearTimeout(timer);
      reject(signal!.reason);
    };
    const timer = setTimeout(() => {
      signal?.removeEventListener("abort", onAbort);
      resolve();
    }, ms);

    signal?.addEventListener("abort", onAbort, { once: true });
  });
}

async function fetchJsonWithChunk(
  uri: string,
  chunk: Blob,
  tryAttempt: number,
  signal?: AbortSignal,
  delayMs = 200,
) {
  let response: Response | undefined;
  let lastError: unknown;

  for (let i = 0; i < tryAttempt; i++) {
    signal?.throwIfAborted();

    try {
      const r = await fetch(uri, { method: "POST", body: chunk, signal });
      if (r.ok) {
        response = r;
        break;
      }
      lastError = new Error(`fetch ${uri} failed: ${r.status}`);
    } catch (err) {
      if (signal?.aborted) throw err; // never retry an abort
      lastError = err; // network error: retry
    }

    if (i < tryAttempt - 1) await sleep(delayMs, signal);
  }

  if (!response) throw lastError;
  return response.json();
}
