import { render } from "preact";
import "./styles/miniserve/miniserve.scss";
import "./styles/miniserve/miniserve_custom.scss";
import "./styles/global.scss";
import { App } from "./app.tsx";

render(<App />, document.getElementById("app")!);
