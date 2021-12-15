import m from "mithril";

import { About, App, NotFound } from "./views";

const root = document.querySelector("#app");
if (root) {
  m.route(root, "/", {
    "/": App,
    "/about": About,
    "/:404": NotFound,
  });
}
