import m from "mithril";

export default class Header {
  view(vnode) {
    return m(
      "header",
      { class: "header" },
      m("nav", [
        m("a", { href: "#!/" }, "Home"),
        m("a", { href: "#!/about" }, "About"),
      ]),
    );
  }
}
