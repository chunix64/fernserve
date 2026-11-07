import {
  type Dispatch,
  type StateUpdater,
  useMemo,
  useState,
} from "preact/hooks";
import type { FileSystemNode } from "@/models";
import { getBaseName } from "@/helpers/path.ts";
import { getSizeString } from "@/helpers/size.ts";
import { getTimeAgoString, humanizeUnixTime } from "@/helpers/date.ts";
import FernClient from "@/api/client.ts";

type FileListProps = {
  fileSystemNodes: FileSystemNode[];
  setFileSystemNodes: Dispatch<StateUpdater<FileSystemNode[]>>;
};

type SortKey = "name" | "size" | "date";
type SortOrder = "asc" | "desc";
type SortState = { key: SortKey; order: SortOrder };

const SORT_LABELS: Record<SortKey, string> = {
  name: "name",
  size: "size",
  date: "date",
};

export default function FileList(
  { fileSystemNodes, setFileSystemNodes }: FileListProps,
) {
  const [sort, setSort] = useState<SortState>({ key: "name", order: "asc" });

  const sortedNodes = useMemo(
    () => sortNodes(fileSystemNodes, sort),
    [fileSystemNodes, sort],
  );

  const toggleSort = (key: SortKey) =>
    setSort((prev) => ({
      key,
      order: prev.key === key && prev.order === "asc" ? "desc" : "asc",
    }));

  const header = (key: SortKey, label: string) => (
    <SortLink
      sortKey={key}
      label={label}
      sort={sort}
      onSort={toggleSort}
    />
  );

  return (
    <table>
      <thead>
        <tr>
          <th class="name">{header("name", "Name")}</th>
          <th class="size">{header("size", "Size")}</th>
          <th class="date">{header("date", "Last modification")}</th>
          <th class="actions">
            <span>Actions</span>
          </th>
        </tr>
      </thead>
      <tbody>
        {
          <tr>
            <td colspan={4}>
              <p>
                <span class="root-chevron">◂</span>
                <a class="root" href="../">Parent directory</a>
              </p>
            </td>
          </tr>
        }
        {getEntries(sortedNodes, sort, toggleSort, setFileSystemNodes)}
      </tbody>
    </table>
  );
}

// Sorting
function sortNodes(nodes: FileSystemNode[], { key, order }: SortState) {
  const dir = order === "asc" ? 1 : -1;

  const byName = (a: FileSystemNode, b: FileSystemNode) =>
    (getBaseName(a.path.replace(/\/+$/, "")) ?? "").localeCompare(
      getBaseName(b.path.replace(/\/+$/, "")) ?? "",
      undefined,
      { numeric: true, sensitivity: "base" },
    );

  return [...nodes].sort((a, b) => {
    // Directories always first
    if (a.nodeType !== b.nodeType) return a.nodeType === "Dir" ? -1 : 1;

    switch (key) {
      case "size": {
        // Dirs have no size: fall back to name ordering among them
        if (a.nodeType === "Dir") return byName(a, b) * dir;
        return ((a.size ?? 0) - (b.size ?? 0)) * dir || byName(a, b);
      }
      case "date":
        return (a.lastModified.getTime() - b.lastModified.getTime()) * dir ||
          byName(a, b);
      case "name":
      default:
        return byName(a, b) * dir;
    }
  });
}

// Sort control (no href, no navigation)
type SortLinkProps = {
  sortKey: SortKey;
  label: string;
  sort: SortState;
  onSort: (key: SortKey) => void;
};

function SortLink({ sortKey, label, sort, onSort }: SortLinkProps) {
  const active = sort.key === sortKey;
  const nextOrder: SortOrder = active && sort.order === "asc" ? "desc" : "asc";
  const nextLabel = nextOrder === "asc" ? "ascending" : "descending";

  return (
    <span class={active ? "active" : ""}>
      <span class="chevron">{active && sort.order === "asc" ? "▴" : "▾"}</span>
      <a
        role="button"
        tabIndex={0}
        title={`Sort by ${SORT_LABELS[sortKey]} in ${nextLabel} order`}
        onClick={() => onSort(sortKey)}
        onKeyDown={(e) => {
          if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            onSort(sortKey);
          }
        }}
      >
        {label}
      </a>
    </span>
  );
}

// Helpers
function getEntries(
  fileSystemNodes: FileSystemNode[],
  sort: SortState,
  onSort: (key: SortKey) => void,
  setFileSystemNodes: Dispatch<StateUpdater<FileSystemNode[]>>,
) {
  return fileSystemNodes.map((fileSystemNode) => {
    const isDir = fileSystemNode.nodeType === "dir";
    const trimmed = isDir
      ? fileSystemNode.path.replace(/\/+$/, "")
      : fileSystemNode.path;
    const baseName = getBaseName(trimmed);
    const name = baseName + (isDir ? "/" : "");
    const href = isDir ? trimmed + "/" : fileSystemNode.path;

    if (isDir && !(baseName === "." || baseName === "..")) {
      return getDir(fileSystemNode, setFileSystemNodes, href, name);
    }

    if (fileSystemNode.nodeType === "file") {
      return getFile(
        fileSystemNode,
        setFileSystemNodes,
        href,
        name,
        sort,
        onSort,
      );
    }
  });
}

function getDir(
  { path, lastModified }: FileSystemNode,
  setFileSystemNodes: Dispatch<StateUpdater<FileSystemNode[]>>,
  href: string,
  name: string,
) {
  return (
    <tr key={path} class="entry-type-directory">
      <td>
        <p>
          <a class="directory" href={href}>{name}</a>
        </p>
      </td>
      <td class="size-cell" data-size="-"></td>
      <td class="date-cell">
        <span>{humanizeUnixTime(lastModified)}</span>
        <span class="history">{getTimeAgoString(lastModified)}</span>
      </td>
      <td class="actions-cell">
        <form
          class="rm_form"
          onSubmit={(e) => {
            e.preventDefault();
            remove(path, setFileSystemNodes);
          }}
        >
          <button
            type="submit"
            title="Delete"
          >
            ✗
          </button>
        </form>
      </td>
    </tr>
  );
}

function getFile(
  { path, size, lastModified }: FileSystemNode,
  setFileSystemNodes: Dispatch<StateUpdater<FileSystemNode[]>>,
  href: string,
  name: string,
  sort: SortState,
  onSort: (key: SortKey) => void,
) {
  const sizeString = getSizeString(size);

  return (
    <tr key={path} class="entry-type-file">
      <td>
        <p>
          <a class="file" href={href}>{name}</a>
          <span class="mobile-info size">
            <SortLink
              sortKey="size"
              label={sizeString}
              sort={sort}
              onSort={onSort}
            />
          </span>
          <span class="mobile-info history">
            <SortLink
              sortKey="date"
              label={getTimeAgoString(lastModified)}
              sort={sort}
              onSort={onSort}
            />
          </span>
        </p>
      </td>
      <td class="size-cell">{sizeString}</td>
      <td class="date-cell">
        <span>{humanizeUnixTime(lastModified)}</span>
        <span class="history">{getTimeAgoString(lastModified)}</span>
      </td>
      <td class="actions-cell">
        <form
          class="rm_form"
          onSubmit={(e) => {
            e.preventDefault();
            remove(path, setFileSystemNodes);
          }}
        >
          <button
            type="submit"
            title="Delete"
          >
            ✗
          </button>
        </form>
      </td>
    </tr>
  );
}

// Events
async function remove(
  path: string,
  setFileSystemNodes: Dispatch<StateUpdater<FileSystemNode[]>>,
) {
  if (await FernClient.remove(path)) {
    setFileSystemNodes((prev) =>
      prev.filter((element) => element.path !== path)
    );
  } else {
    console.error("Failed to remove directory: ", path);
  }
}
