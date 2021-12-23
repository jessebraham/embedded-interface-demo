import m from "mithril";

const ChipInfo = {
  model: null,
  revision: null,
  cores: null,
  features: [],

  load: () => {
    m.request({
      method: "GET",
      url: "/api/info",
    }).then(({ data: { model, revision, cores, features } }) => {
      ChipInfo.model = model;
      ChipInfo.revision = revision;
      ChipInfo.cores = cores;
      ChipInfo.features = features;
    });
  },
};

export default ChipInfo;
