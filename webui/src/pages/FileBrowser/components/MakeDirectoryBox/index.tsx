export default function MakeDirectoryBox() {
  return (
    <form
      class="tool"
      id="mkdir"
      data-tool="mkdir"
      action="/upload?path=/test"
      method="POST"
      enctype="multipart/form-data"
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
        <button type="submit" title="Create directory">
          Create directory
        </button>
      </div>
    </form>
  );
}
