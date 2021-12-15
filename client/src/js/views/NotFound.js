import m from "mithril";

import { Header } from "../components";

export default class NotFound {
  constructor() {
    document.title = "Page Not Found";
  }

  view(vnode) {
    return m("", {}, [m(Header)]);
  }
}
