import { render } from "preact";
import "./styles/index.scss";
import "./styles/app.scss";
import { App } from "./app.tsx";

render(<App />, document.getElementById("app")!);
