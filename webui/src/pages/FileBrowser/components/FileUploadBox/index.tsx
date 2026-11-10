import FernClient from "@/api/client.ts";
import type { Dispatch, StateUpdater } from "preact/hooks";
import type { FileSystemNode } from "@/models";
import type { TargetedSubmitEvent } from "preact";
import { isValidName, joinPath } from "@/helpers/path";

type FileUploadBoxProps = {
  setFileSystemNodes: Dispatch<StateUpdater<FileSystemNode[]>>;
};

export default function FileUploadBox(
  { setFileSystemNodes }: FileUploadBoxProps,
) {
  return (
    <form
      class="tool"
      id="file_submit"
      data-tool="upload"
      enctype="multipart/form-data"
      onSubmit={async (e) => {
        uploadFile(e, setFileSystemNodes);
      }}
    >
      <p>Select a file to upload or drag it anywhere into the window</p>
      <div>
        <input
          id="file-input"
          type="file"
          name="file_to_upload"
          required
          multiple
        >
        </input>
        <button type="submit" title="Upload File">Upload file</button>
      </div>
    </form>
  );
}

async function uploadFile(
  e: TargetedSubmitEvent<HTMLFormElement>,
  setFileSystemNodes: Dispatch<StateUpdater<FileSystemNode[]>>,
) {
  e.preventDefault();
  const form = e.currentTarget;
  const formData = new FormData(form);
  const files = formData.getAll("file_to_upload");
  files.forEach(async (file) => {
    if (file instanceof File) {
      const name = file instanceof File ? file.name : "";
      if (!name || !isValidName(name)) return;

      await uploadFileInner(
        joinPath(location.pathname, name),
        file,
        setFileSystemNodes,
      );
    }
  });

  form.reset();
}

async function uploadFileInner(
  path: string,
  file: File,
  setFileSystemNodes: Dispatch<StateUpdater<FileSystemNode[]>>,
) {
  if (await FernClient.uploadFile(path, file)) {
    setFileSystemNodes((prev: FileSystemNode[]) => [...prev, {
      path: path,
      nodeType: "file",
      size: file instanceof File ? file.size : 0,
      lastModified: new Date(Date.now()),
    }]);
  } else {
    console.error("Failed to upload file: ", path);
  }
}
