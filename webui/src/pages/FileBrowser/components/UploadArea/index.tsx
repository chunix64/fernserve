import type { UploadFile } from "@/hooks/useUploadQueue";
import { getSizeString } from "@/helpers/size.ts";

interface Props {
  queue: UploadFile[];
  onCancel: (id: number) => void;
  onCancelAll: () => void;
  onClose: () => void;
}

function percentOf(item: UploadFile): number {
  if (item.state === "Completed") return 100;
  if (item.totalBytes === 0) return 0;
  return Math.floor((item.uploadedBytes / item.totalBytes) * 100);
}

const STATE_CLASS = {
  Pending: "pending",
  Uploading: "uploading",
  Completed: "complete",
  Failed: "failed",
  Cancelled: "cancelled",
} as const;

const ICONS = {
  Pending: "✖",
  Uploading: "✖",
  Completed: "✔",
  Failed: "⚠",
  Cancelled: "—",
} as const;

function UploadItem(
  { item, onCancel }: { item: UploadFile; onCancel: (id: number) => void },
) {
  const state = STATE_CLASS[item.state];
  const active = item.state === "Pending" || item.state === "Uploading";
  const label = item.state === "Failed" || item.state === "Cancelled"
    ? item.state
    : `${percentOf(item)}%`;

  return (
    <li class="upload_file_item" data-state={state}>
      <div class="upload_file_container">
        <div class={`upload_file_text ${state}`}>
          <span class="file_upload_percent">{label}</span> -{" "}
          <span class="file_size">{getSizeString(item.totalBytes)}</span> -{" "}
          <span class="file_name">{item.file.name}</span>
        </div>
        <button
          class={`file_cancel_upload ${state}`}
          disabled={!active}
          aria-label={`Cancel upload of ${item.file.name}`}
          onClick={() => onCancel(item.id)}
        >
          {ICONS[item.state]}
        </button>
      </div>
      <div
        class={`file_progress_bar ${state}`}
        style={{ width: `${percentOf(item)}%` }}
      />
    </li>
  );
}

export default function UploadArea(
  { queue, onCancel, onCancelAll, onClose }: Props,
) {
  const total = queue.length;
  const completed = queue.filter((i) => i.state === "Completed").length;
  const cancelled = queue.filter((i) => i.state === "Cancelled").length;
  const failed = queue.filter((i) => i.state === "Failed").length;
  const active =
    queue.filter((i) => i.state === "Pending" || i.state === "Uploading")
      .length;

  const title = active > 0
    ? `Uploading ${active} file${active === 1 ? "" : "s"}…`
    : failed > 0
    ? "Finished with errors"
    : completed === 0 && cancelled > 0
    ? "Cancelled"
    : "Complete!";

  const actionText = `Complete ${completed}/${total}` +
    (failed > 0 ? ` · ${failed} failed` : "");

  return (
    <div
      class={`upload_area ${total > 0 ? "active" : ""}`}
      id="upload_area"
    >
      <div class="upload_container">
        <div class="upload_header">
          <h4 style="margin:0px" id="upload_title">{title}</h4>
          <svg
            id="upload-toggle"
            class="size-6"
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            stroke-width="1.5"
            stroke="currentColor"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              d="m4.5 15.75 7.5-7.5 7.5 7.5"
            />
          </svg>
        </div>

        {
          <>
            <div class="upload_action">
              <p id="upload_action_text">{actionText}</p>
              <button
                class="upload_cancel"
                id="upload_cancel"
                onClick={active > 0 ? onCancelAll : onClose}
              >
                {active > 0 ? "CANCEL" : "CLOSE"}
              </button>
            </div>
            <div class="upload_files">
              <ul class="upload_file_list" id="upload_file_list">
                {queue.map((item) => (
                  <UploadItem key={item.id} item={item} onCancel={onCancel} />
                ))}
              </ul>
            </div>
          </>
        }
      </div>
    </div>
  );
}
