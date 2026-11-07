function getPath() {
  const paths = location.pathname.split("/").filter(Boolean);

  if (paths.length === 0) {
    return;
  }

  return paths.map((path, index) => {
    const isLast = index === paths.length - 1;
    const href = "/" + paths.slice(0, index + 1).join("/") + "/";

    return (
      <>
        {isLast
          ? (
            <span>
              <bdi>{decodeURIComponent(path)}</bdi>
            </span>
          )
          : (
            <a href={href}>
              <bdi>{decodeURIComponent(path)}</bdi>
            </a>
          )}
        /
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
