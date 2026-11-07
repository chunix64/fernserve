export function pathComponents(path: string): string[] {
  const result: string[] = [];

  if (
    path === "" || path === "/" || path === "/." || path === "./" ||
    path === "./."
  ) {
    return ["/"];
  }

  for (const c of path.split("/").filter((c) => c !== "" && c !== ".")) {
    if (c === "..") {
      result.pop();
    } else {
      result.push(c);
    }
  }

  return result;
}

export function getBaseName(path: string) {
  return path.split("/").pop();
}
