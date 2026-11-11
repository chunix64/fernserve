import { useEffect, useState } from "preact/hooks";

import FernClient from "@/api/client.ts";
import Title from "@/pages/FileBrowser/components/Title";
import SearchBox from "@/pages/FileBrowser/components/SearchBox";
import FileUploadBox from "@/pages/FileBrowser/components/FileUploadBox";
import CreateDirectoryBox from "@/pages/FileBrowser/components/CreateDirectoryBox";
import FileList from "@/pages/FileBrowser/components/FileList";
import type { FileSystemNode } from "@/models";

export default function FileBrowser() {
  const dir = location.pathname || "/";
  const [fileSystemNodes, setFileSystemNodes] = useState<FileSystemNode[]>([]);

  // FileBrowser
  useEffect(() => {
    FernClient.readDir(dir)
      .then((nodes) => {
        setFileSystemNodes(nodes);
      })
      .catch(console.error);
  }, [dir]);

  return (
    <>
      <span id="top"></span>
      <div class="title-search-box">
        <Title />
        <SearchBox />
      </div>
      <div class="toolbar">
        <div class="tool_row upload_tools">
          <FileUploadBox setFileSystemNodes={setFileSystemNodes} />
          <CreateDirectoryBox setFileSystemNodes={setFileSystemNodes} />
        </div>
      </div>
      <FileList
        fileSystemNodes={fileSystemNodes}
        setFileSystemNodes={setFileSystemNodes}
      />
    </>
  );
}
