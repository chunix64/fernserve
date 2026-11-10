import type { FileSystemNode } from "@/models";
import FernClient from "@/api/client.ts";
import type { Dispatch, StateUpdater } from "preact/hooks";
import { isValidName, joinPath } from "@/helpers/path";
import type { TargetedSubmitEvent } from "preact";

type CreateDirectoryBoxProps = {
  setFileSystemNodes: Dispatch<StateUpdater<FileSystemNode[]>>;
};

export default function CreateDirectoryBox(
  { setFileSystemNodes }: CreateDirectoryBoxProps,
) {
  return (
    <form
      class="tool"
      id="mkdir"
      data-tool="mkdir"
      enctype="multipart/form-data"
      onSubmit={async (e) => {
        createDirectory(e, setFileSystemNodes);
      }}
    >
      <p>Specify a directory name to create</p>
      <div>
        <input
          type="text"
          name="mkdir"
          required
          placeholder="Directory name"
        >
        </input>
        <button
          type="submit"
          title="Create directory"
        >
          Create directory
        </button>
      </div>
    </form>
  );
}

// Events
async function createDirectory(
  e: TargetedSubmitEvent<HTMLFormElement>,
  setFileSystemNodes: Dispatch<StateUpdater<FileSystemNode[]>>,
) {
  e.preventDefault();
  const form = e.currentTarget;
  const name = (new FormData(form).get("mkdir") as string)?.trim();
  if (!name || !isValidName(name)) return;
  await createDirectoryInner(
    joinPath(location.pathname, name),
    setFileSystemNodes,
  );
  form.reset();
}

async function createDirectoryInner(
  path: string,
  setFileSystemNodes: Dispatch<StateUpdater<FileSystemNode[]>>,
) {
  if (await FernClient.createDirectory(path)) {
    setFileSystemNodes((prev: FileSystemNode[]) => [...prev, {
      path: path,
      nodeType: "dir",
      size: 0,
      lastModified: new Date(Date.now()),
    }]);
  } else {
    console.error("Failed to create directory: ", path);
  }
}
