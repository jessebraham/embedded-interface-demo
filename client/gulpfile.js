// ---------------------------------------------------------------------------
// Imports

// Since Gulp is a CommonJS module, we have to stick-handle around this a bit.
import gulp from "gulp";
const { parallel, series, src, dest, task, watch } = gulp;

import del from "del";
import esbuild from "gulp-esbuild";
import gzip from "gulp-gzip";
import htmlmin from "gulp-htmlmin";
import inlineSource from "gulp-inline-source";
import nesting from "tailwindcss/nesting/index.js";
import postcss from "gulp-postcss";
import postcssImport from "postcss-import";
import size from "gulp-size";
import tailwindcss from "tailwindcss";

// We cannot use ES6 modules for the TailwindCSS configuration file, so we
// instead import it and pass it directly to the `tailwindcss` module.
import tailwindConfig from "./tailwind.config.js";

// ---------------------------------------------------------------------------
// Constants

const Globs = Object.freeze({
  DIST: "./dist/*",
  HTML: "./src/**/*.html",
  SCRIPTS: "./src/js/**/*.js",
  STYLES: "./src/css/**/*.css",
});

const Paths = Object.freeze({
  DIST: "./dist",
  HTML: "./src/index.html",
  SCRIPTS: "./src/js/index.js",
  STYLES: "./src/css/main.css",
});

// ---------------------------------------------------------------------------
// Build Functions

async function clean() {
  await del([Globs.DIST]);
}

function buildCss() {
  return src(Paths.STYLES)
    .pipe(postcss([postcssImport(), nesting(), tailwindcss(tailwindConfig)]))
    .pipe(dest(Paths.DIST));
}

function buildJs() {
  return src(Paths.SCRIPTS)
    .pipe(
      esbuild({
        bundle: true,
        loader: { ".svg": "text" },
        sourcemap: process.env.NODE_ENV !== "production",
      }),
    )
    .pipe(dest(Paths.DIST));
}

function combine(done) {
  const compress = process.env.NODE_ENV === "production";
  let stream = src(Paths.HTML).pipe(inlineSource({ compress }));

  if (compress) {
    stream = stream
      .pipe(htmlmin({ collapseWhitespace: true }))
      .pipe(dest(Paths.DIST))
      .pipe(gzip());
  }

  stream
    .pipe(dest(Paths.DIST))
    .pipe(size({ pretty: true }))
    .pipe(size({ pretty: true, gzip: true }));

  done();
}

function buildAndCombine(done, production = false) {
  if (production) {
    process.env["NODE_ENV"] = "production";
  }

  series(clean, parallel(buildCss, buildJs), combine)(done);
}

// ---------------------------------------------------------------------------
// Tasks

task("default", buildAndCombine);

task("build:dev", buildAndCombine);

task("build:prod", (done) => buildAndCombine(done, true));

task("watch", () => {
  watch([Globs.HTML, Globs.SCRIPTS, Globs.STYLES]).on(
    "change",
    buildAndCombine,
  );
});
