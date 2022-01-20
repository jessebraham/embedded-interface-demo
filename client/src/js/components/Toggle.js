import m from "mithril";

import MoonSvg from "../../img/moon.svg";
import SunSvg from "../../img/sun.svg";

const Moon = m.trust(MoonSvg);
const Sun = m.trust(SunSvg);

export default class Toggle {
  view() {
    return m("label", { for: "toggle", class: "toggle" }, [
      m("div", { class: "toggle-label" }, Moon),
      m("div", { class: "toggle-input" }, [
        m("input", {
          id: "toggle",
          type: "checkbox",
          onchange: onToggleChange,
        }),
        m("div", { class: "line" }),
        m("div", { class: "dot" }),
      ]),
      m("div", { class: "toggle-label" }, Sun),
    ]);
  }
}

function onToggleChange({ target: { checked } }) {
  m.request({
    method: "GET",
    url: `/api/light/${checked ? "on" : "off"}`,
  })
    .then(() => {
      if (checked) {
        document.body.classList.remove("dark");
      } else {
        document.body.classList.add("dark");
      }
    })
    .catch(() => {
      target.checked = !checked;
    });
}
