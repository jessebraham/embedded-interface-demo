import m from "mithril";

import { Header } from "../components";
import { DeviceInfo } from "../models";

export default class About {
  constructor() {
    document.title = "About";
  }

  view(vnode) {
    return m("", {}, [m(Header)]);
  }
}
