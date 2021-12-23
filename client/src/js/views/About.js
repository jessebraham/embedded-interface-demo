import m from "mithril";

import { Header } from "../components";
import { ChipInfo } from "../models";

export default class About {
  constructor() {
    document.title = "About";
    ChipInfo.load();
  }

  view() {
    return m("div", { class: "layout" }, [
      m(Header),
      m(
        "div",
        { class: "content" },
        m("table", [
          m("tr", [
            m("td", "Chip:"),
            m("td", `${ChipInfo.model} (revision ${ChipInfo.revision})`),
          ]),
          m("tr", [m("td", "Cores:"), m("td", ChipInfo.cores)]),
          m("tr", [
            m("td", "Features:"),
            m("td", ChipInfo.features.join(", ")),
          ]),
        ]),
      ),
    ]);
  }
}
