export default function FileUploadBox() {
  return (
    <form
      class="tool"
      id="file_submit"
      data-tool="upload"
      action="/upload?path=/test"
      method="POST"
      enctype="multipart/form-data"
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
