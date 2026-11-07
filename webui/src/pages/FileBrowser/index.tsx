import { useEffect, useState } from "preact/hooks";

import FernClient from "@/api/client.ts";
import Title from "@/pages/FileBrowser/components/Title";
import SearchBox from "@/pages/FileBrowser/components/SearchBox";
import FileUploadBox from "@/pages/FileBrowser/components/FileUploadBox";
import MakeDirectoryBox from "@/pages/FileBrowser/components/MakeDirectoryBox";
import FileList from "@/pages/FileBrowser/components/FileList";
import type { FileSystemNode } from "@/models";

export default function FileBrowser() {
  let [fileSystemNodes, setFileSystemNodes] = useState<FileSystemNode[]>([]);

  // FileBrowser
  useEffect(() => {
    let cancelled = false;
    FernClient.readDir("")
      .then((nodes) => {
        if (!cancelled) setFileSystemNodes(nodes);
      })
      .catch(console.error);
    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <>
      <span id="top"></span>
      <div class="title-search-box">
        <Title />
        <SearchBox />
      </div>
      <div class="toolbar">
        <div class="tool_row upload_tools">
          <FileUploadBox />
          <MakeDirectoryBox />
        </div>
      </div>
      <FileList fileSystemNodes={fileSystemNodes} />
    </>
  );
}
