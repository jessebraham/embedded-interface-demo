import m from "mithril";

import { Header, Toggle } from "../components";

export default class App {
  constructor() {
    document.title = "Embedded Interface Demo";
  }

  view(vnode) {
    return m("div", { class: "layout" }, [
      m(Header),
      m("div", { class: "content" }, m(Toggle)),
    ]);
  }
}
