import type { TargetedSubmitEvent } from "preact";

type FileUploadBoxProps = {
  enqueue: (files: File[]) => void;
};

export default function FileUploadBox({ enqueue }: FileUploadBoxProps) {
  function onSubmit(e: TargetedSubmitEvent<HTMLFormElement>) {
    e.preventDefault();
    const form = e.currentTarget;
    const files = new FormData(form)
      .getAll("file_to_upload")
      .filter((item): item is File => item instanceof File);
    form.reset();
    enqueue(files);
  }

  return (
    <form
      class="tool"
      id="file_submit"
      data-tool="upload"
      enctype="multipart/form-data"
      onSubmit={onSubmit}
    >
      <p>Select a file to upload or drag it anywhere into the window</p>
      <div>
        <input
          id="file-input"
          type="file"
          name="file_to_upload"
          required
          multiple
        />
        <button type="submit" title="Upload File">Upload file</button>
      </div>
    </form>
  );
}
