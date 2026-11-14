import FernClient from "@/api/client.ts";
import { isValidName, joinPath } from "@/helpers/path";
import type { FileSystemNode } from "@/models";
import { useRef, useState } from "preact/hooks";
import type { Dispatch, StateUpdater } from "preact/hooks";

export type UploadState =
  | "Pending"
  | "Uploading"
  | "Completed"
  | "Failed"
  | "Cancelled";

export interface UploadFile {
  id: number;
  file: File;
  path: string;
  state: UploadState;
  uploadedBytes: number;
  totalBytes: number;
}

const isActive = (s: UploadState) => s === "Pending" || s === "Uploading";

let nextId = 0;

export function useUploadQueue(
  setFileSystemNodes: Dispatch<StateUpdater<FileSystemNode[]>>,
) {
  const [queue, setQueue] = useState<UploadFile[]>([]);
  const jobs = useRef<UploadFile[]>([]);
  const controllers = useRef(new Map<number, AbortController>());
  const running = useRef(false);

  const patch = (id: number, p: Partial<UploadFile>) =>
    setQueue((prev) => prev.map((i) => (i.id === id ? { ...i, ...p } : i)));

  async function runJob(job: UploadFile) {
    const controller = controllers.current.get(job.id);
    if (!controller || controller.signal.aborted) return;

    patch(job.id, { state: "Uploading" });

    let ok = false;
    try {
      ok = await FernClient.uploadFile(
        job.path,
        job.file,
        (uploadedBytes) => patch(job.id, { uploadedBytes }),
        controller.signal,
      );
    } catch {
      ok = false;
    }

    if (ok) {
      setFileSystemNodes((prev) => [...prev, {
        path: job.path,
        nodeType: "file",
        size: job.file.size,
        lastModified: new Date(),
      }]);
      patch(job.id, { state: "Completed", uploadedBytes: job.file.size });
    } else {
      patch(job.id, {
        state: controller.signal.aborted ? "Cancelled" : "Failed",
      });
    }
    controllers.current.delete(job.id);
  }

  // Single worker: files upload one at a time, even across multiple submits.
  async function drain() {
    if (running.current) return;
    running.current = true;
    try {
      let job: UploadFile | undefined;
      while ((job = jobs.current.shift())) await runJob(job);
    } finally {
      running.current = false;
    }
  }

  function enqueue(files: File[]) {
    const dir = location.pathname;
    const items = files.map((file): UploadFile => ({
      id: nextId++,
      file,
      path: joinPath(dir, file.name),
      state: file.name && isValidName(file.name) ? "Pending" : "Failed",
      uploadedBytes: 0,
      totalBytes: file.size,
    }));

    setQueue((prev) => [...prev, ...items]);

    for (const item of items) {
      if (item.state !== "Pending") continue;
      controllers.current.set(item.id, new AbortController());
      jobs.current.push(item);
    }
    void drain();
  }

  function cancel(id: number) {
    controllers.current.get(id)?.abort();
    setQueue((prev) =>
      prev.map((i) =>
        i.id === id && isActive(i.state) ? { ...i, state: "Cancelled" } : i
      )
    );
  }

  function cancelAll() {
    jobs.current = [];
    controllers.current.forEach((c) => c.abort());
    setQueue((prev) =>
      prev.map((i) => (isActive(i.state) ? { ...i, state: "Cancelled" } : i))
    );
  }

  function clear() {
    setQueue([]);
  }

  return { queue, enqueue, cancel, cancelAll, clear };
}
