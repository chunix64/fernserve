function getPath() {
  let paths = location.pathname.split("/").slice(1);

  if (paths[paths.length - 1] === "") {
    paths.pop();
  }

  if (paths[0]?.length < 1) {
    return;
  }

  return paths.map((path) => {
    return (
      <>
        <span>
          <bdi>{path}</bdi>
        </span>/
      </>
    );
  });
}

export default function Title() {
  return (
    <h1 class="title" dir="ltr">
      <a href="/">
        <bdi>{location.host}</bdi>
      </a>/
      {getPath()}
    </h1>
  );
}
