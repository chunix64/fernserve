// TODO: SPA search

export default function SearchBox() {
  return (
    <div class="search-box">
      <form id="search" method="GET">
        <input type="text" name="search" value="" placeholder="Search...">
        </input>
        <button type="submit">Search</button>
      </form>
    </div>
  );
}
