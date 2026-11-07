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

export function joinPath(dir: string, name: string): string {
  const base = decodeURIComponent(dir).replace(/\/+$/, "");
  return `${base}/${name}`;
}

export function isValidName(name: string): boolean {
  return (
    name.length > 0 &&
    name !== "." &&
    name !== ".." &&
    !/[\/\\\0]/.test(name)
  );
}
