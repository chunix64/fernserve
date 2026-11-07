import { render } from "preact";
import "./styles/miniserve/miniserve.scss";
import "./styles/miniserve/miniserve_custom.scss";
import "./styles/index.scss";
import { App } from "./app.tsx";

render(<App />, document.getElementById("app")!);
